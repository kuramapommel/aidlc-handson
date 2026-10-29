//! 受け口の中身。データの写し（`StoreState`）と要求の値から応答を作る関数。
//! 業務の判断は業務の部品の純粋関数に任せ、ここでは入力の検証と形の変換だけを行う。

use axum::Json;
use chrono::{DateTime, Utc};
use course_catalog::{
    CourseId, SessionId, course_detail as catalog_detail, find_session_with_course,
    list_courses as catalog_list,
};
use eligibility::{
    EligibilityResult, EmployeeId, company_finder, determine_eligibility, employee_finder,
    switchable_employees,
};
use in_memory_store::StoreState;
use purchasing::{
    PurchaseApproval, PurchaseDecision, TicketId, TicketView, find_own_ticket, issued_counter,
    issued_counts, purchased_session_ids, session_availability, ticket_view,
};
use serde::de::DeserializeOwned;
use training_budget::{EmployeeRef, balance_of};

use crate::dto::{
    CourseDetailDto, CourseDto, CourseListDto, CourseRowDto, EmployeeListDto, MeDto, PurchaseBody,
    PurchaseResultDto, SessionDto, SwitchEmployeeBody, SwitchableEmployeeDto, TicketDto,
};
use crate::errors::ApiError;

/// 本文の JSON を読む。読めなければ 400（NFR3.1）。
fn parse_body<T: DeserializeOwned>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body)
        .map_err(|_| ApiError::BadRequest("本文の JSON を読み取れません".to_owned()))
}

/// `GET /api/courses`: 探索条件なしのコース一覧（既定の並び。BR3.3）。
pub fn list_courses(state: &StoreState, now: DateTime<Utc>) -> Json<CourseListDto> {
    let counts = issued_counts(&state.tickets);
    let rows = catalog_list(
        &state.courses,
        &state.sessions,
        issued_counter(&counts),
        now,
    );
    let courses: Vec<CourseRowDto> = rows.iter().map(CourseRowDto::from).collect();
    Json(CourseListDto {
        count: courses.len(),
        courses,
    })
}

/// `GET /api/courses/{courseId}`: コースの詳細と、表示用の状態付きの日程（BR2.1〜BR2.4）。
pub fn course_detail(
    state: &StoreState,
    raw_course_id: &str,
    employee_id: Option<&EmployeeId>,
    now: DateTime<Utc>,
) -> Result<Json<CourseDetailDto>, ApiError> {
    let course_id =
        CourseId::parse(raw_course_id).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let counts = issued_counts(&state.tickets);
    let detail = catalog_detail(
        &state.courses,
        &state.sessions,
        &course_id,
        issued_counter(&counts),
        now,
    )
    .ok_or(ApiError::NotFound("コース"))?;
    let purchased = purchased_session_ids(&state.tickets, employee_id);
    let sessions = detail
        .sessions
        .iter()
        .map(|v| {
            let status = session_availability(v.status, purchased.contains(&v.session.id));
            SessionDto::new(&v.session, v.remaining_seats, status)
        })
        .collect();
    Ok(Json(CourseDetailDto {
        course: CourseDto::from(&detail.course),
        sessions,
    }))
}

/// 操作中の従業員の情報（表示名・企業名・契約状態・利用資格・残高）。
pub fn me(state: &StoreState, employee_id: Option<&EmployeeId>) -> MeDto {
    let find_employee = employee_finder(&state.employees);
    let find_company = company_finder(&state.companies);
    let found = employee_id
        .and_then(&find_employee)
        .and_then(|e| find_company(&e.company_id).map(|c| (e, c)));
    let Some((employee, company)) = found else {
        return MeDto::not_found(employee_id.map(|id| id.as_str().to_owned()));
    };
    let eligibility: EligibilityResult =
        determine_eligibility(Some(&employee.id), &find_employee, &find_company);
    MeDto {
        employee_id: Some(employee.id.as_str().to_owned()),
        balance: balance_of(
            &EmployeeRef(employee.id.as_str().to_owned()),
            &state.budget_entries,
        ),
        display_name: Some(employee.display_name),
        company_name: Some(company.name),
        contract_status: Some(company.contract_status.as_str()),
        eligibility: eligibility.as_str(),
    }
}

/// Cookie がないときに操作中にする、初期データの先頭の従業員（BR6.2）。
pub fn default_employee(state: &StoreState) -> Option<EmployeeId> {
    state.employees.first().map(|e| e.id.clone())
}

/// `GET /api/employees`: 切り替え用の従業員一覧（FR1.2）。
pub fn employees(state: &StoreState) -> Json<EmployeeListDto> {
    let employees = switchable_employees(&state.employees, &state.companies)
        .iter()
        .map(SwitchableEmployeeDto::from)
        .collect();
    Json(EmployeeListDto { employees })
}

/// `POST /api/session/employee` の本文から、切り替え先の従業員を決める。
/// 形式不正は 400、切り替え用の一覧にいなければ 404（BR6.1）。
pub fn switch_target(state: &StoreState, body: &[u8]) -> Result<EmployeeId, ApiError> {
    let parsed: SwitchEmployeeBody = parse_body(body)?;
    let id =
        EmployeeId::parse(&parsed.employee_id).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    switchable_employees(&state.employees, &state.companies)
        .into_iter()
        .any(|e| e.employee_id == id)
        .then_some(id)
        .ok_or(ApiError::NotFound("従業員"))
}

/// `POST /api/purchases` の本文から日程 ID を読む。金額などほかの項目は読まない（BR5.2）。
pub fn purchase_session_id(body: &[u8]) -> Result<SessionId, ApiError> {
    let parsed: PurchaseBody = parse_body(body)?;
    SessionId::parse(&parsed.session_id).map_err(|e| ApiError::BadRequest(e.to_string()))
}

fn view_of(state: &StoreState, ticket: &purchasing::Ticket) -> Option<TicketView> {
    ticket_view(ticket, employee_finder(&state.employees), |sid| {
        find_session_with_course(&state.courses, &state.sessions, sid)
            .map(|(s, c)| (s.clone(), c.clone()))
    })
}

/// 購入の決定を応答にする。承認なら e チケットと購入後の残高、拒否なら 409 と理由（BR5.1）。
pub fn purchase_result(
    state: &StoreState,
    decision: &PurchaseDecision,
) -> Result<Json<PurchaseResultDto>, ApiError> {
    match decision {
        PurchaseDecision::Rejected(reason) => Err(ApiError::Rejected(*reason)),
        PurchaseDecision::Approved(approval) => {
            let PurchaseApproval {
                ticket,
                balance_after,
                ..
            } = approval.as_ref();
            let view = view_of(state, ticket).ok_or(ApiError::Internal)?;
            Ok(Json(PurchaseResultDto {
                ticket: TicketDto::from(&view),
                balance_after: *balance_after,
            }))
        }
    }
}

/// `GET /api/tickets/{ticketId}`: 本人のチケットだけを返す。他人のものは 404（BR5.10、NFR3.7）。
pub fn ticket(
    state: &StoreState,
    raw_ticket_id: &str,
    employee_id: Option<&EmployeeId>,
) -> Result<Json<TicketDto>, ApiError> {
    let ticket_id =
        TicketId::parse(raw_ticket_id).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let ticket = find_own_ticket(&state.tickets, &ticket_id, employee_id)
        .ok_or(ApiError::NotFound("チケット"))?;
    let view = view_of(state, ticket).ok_or(ApiError::Internal)?;
    Ok(Json(TicketDto::from(&view)))
}
