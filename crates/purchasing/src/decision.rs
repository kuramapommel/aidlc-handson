//! 購入の決定関数（ADR-004、BR5.1〜BR5.3、BR5.7、BR4.5）。
//!
//! 入力の写し（[`PurchaseSnapshot`]）と、承認したときに使うチケットの識別子だけから
//! 「承認（チケット 1 枚＋消費の取引 1 件）」か「拒否の理由 1 つ」を返す純粋関数。
//! 時計や乱数は呼ばない（現在時刻は写しに、乱数で作った番号は識別子に入っている）。

use course_catalog::{is_closed, remaining_seats};
use eligibility::{EligibilityResult, EmployeeId};
use training_budget::{
    EmployeeRef, EntryId, PaymentCheck, TicketRef, check_payment, consumption_entry,
};

use crate::model::{
    PurchaseApproval, PurchaseDecision, PurchaseSnapshot, RejectionReason, Ticket, TicketIdentity,
};

/// BR5.1 の順に調べ、最初に当てはまった拒否の理由を返す。当てはまらなければ `None`。
pub fn first_rejection(snapshot: &PurchaseSnapshot) -> Option<RejectionReason> {
    match snapshot.eligibility {
        EligibilityResult::EmployeeNotFound => return Some(RejectionReason::EmployeeNotFound),
        EligibilityResult::CompanyNotActive => return Some(RejectionReason::CompanyNotActive),
        EligibilityResult::Eligible => {}
    }
    let Some((session, course)) = &snapshot.session else {
        return Some(RejectionReason::SessionNotFound);
    };
    if is_closed(session, snapshot.now) {
        return Some(RejectionReason::Closed);
    }
    if snapshot.already_purchased {
        return Some(RejectionReason::AlreadyPurchased);
    }
    if remaining_seats(session, snapshot.issued) == 0 {
        return Some(RejectionReason::Full);
    }
    match check_payment(snapshot.balance, course.fee_points) {
        PaymentCheck::Insufficient { .. } => Some(RejectionReason::InsufficientBalance),
        PaymentCheck::Payable { .. } => None,
    }
}

/// 消費の取引の ID を、チケット ID から決める（チケット 1 枚に消費の取引がちょうど 1 件）。
pub fn consumption_entry_id(identity: &TicketIdentity) -> EntryId {
    EntryId(format!("consume-{}", identity.ticket_id.as_str()))
}

/// 購入を決定する。
///
/// 支払うポイントはコースの受講料（BR5.2）、購入日時は写しの現在時刻（BR5.7）。
pub fn decide_purchase(snapshot: &PurchaseSnapshot, identity: TicketIdentity) -> PurchaseDecision {
    if let Some(reason) = first_rejection(snapshot) {
        return PurchaseDecision::Rejected(reason);
    }
    match approve(snapshot, identity) {
        Some(approval) => PurchaseDecision::Approved(Box::new(approval)),
        // first_rejection を通った写しでは起きないが、整合しない写しは拒否として扱う
        None => PurchaseDecision::Rejected(RejectionReason::EmployeeNotFound),
    }
}

fn approve(snapshot: &PurchaseSnapshot, identity: TicketIdentity) -> Option<PurchaseApproval> {
    let employee_id: &EmployeeId = snapshot.employee_id.as_ref()?;
    let (session, course) = snapshot.session.as_ref()?;
    let fee = course.fee_points;
    let consumption = consumption_entry(
        consumption_entry_id(&identity),
        EmployeeRef(employee_id.as_str().to_owned()),
        TicketRef(identity.ticket_id.as_str().to_owned()),
        fee,
        snapshot.balance,
        snapshot.now,
    )?;
    let ticket = Ticket {
        id: identity.ticket_id,
        number: identity.ticket_number,
        employee_id: employee_id.clone(),
        session_id: session.id.clone(),
        paid_points: fee,
        purchased_at: snapshot.now,
    };
    Some(PurchaseApproval {
        ticket,
        consumption,
        balance_after: snapshot.balance - i64::from(fee),
    })
}
