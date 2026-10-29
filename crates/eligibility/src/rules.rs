//! 利用資格の判定（BR1.1）と、切り替え用の従業員一覧（FR1.2）。
//!
//! 従業員と企業の取得は、呼び出し側から関数として受け取る（データストアに依存しないため）。

use crate::model::{Company, CompanyId, ContractStatus, Employee, EmployeeId};

/// 利用資格（entities.md の derived_values の eligibility）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EligibilityResult {
    /// 購入できる。
    Eligible,
    /// 所属企業が契約中でない。
    CompanyNotActive,
    /// 従業員 ID に当たる従業員がいない。
    EmployeeNotFound,
}

impl EligibilityResult {
    /// Web API などで使う値。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::CompanyNotActive => "company-not-active",
            Self::EmployeeNotFound => "employee-not-found",
        }
    }
}

/// 利用資格を判定する（BR1.1）。
///
/// 従業員がいなければ employee-not-found、所属企業が契約中でなければ（企業が見つからない
/// 場合も含む）company-not-active、それ以外は eligible。
pub fn determine_eligibility<FE, FC>(
    employee_id: Option<&EmployeeId>,
    find_employee: FE,
    find_company: FC,
) -> EligibilityResult
where
    FE: Fn(&EmployeeId) -> Option<Employee>,
    FC: Fn(&CompanyId) -> Option<Company>,
{
    let Some(employee) = employee_id.and_then(&find_employee) else {
        return EligibilityResult::EmployeeNotFound;
    };
    match find_company(&employee.company_id) {
        Some(company) if company.contract_status == ContractStatus::Active => {
            EligibilityResult::Eligible
        }
        _ => EligibilityResult::CompanyNotActive,
    }
}

/// 切り替え用の従業員 1 名分（entities.md の SwitchableEmployee）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchableEmployee {
    pub employee_id: EmployeeId,
    pub display_name: String,
    pub company_name: String,
    pub contract_status: ContractStatus,
}

/// 切り替え用の従業員一覧を、初期データの順（従業員の並び順）のまま返す。
/// 所属企業が見つからない従業員は一覧に含めない。
pub fn switchable_employees(
    employees: &[Employee],
    companies: &[Company],
) -> Vec<SwitchableEmployee> {
    employees
        .iter()
        .filter_map(|e| {
            let company = companies.iter().find(|c| c.id == e.company_id)?;
            Some(SwitchableEmployee {
                employee_id: e.id.clone(),
                display_name: e.display_name.clone(),
                company_name: company.name.clone(),
                contract_status: company.contract_status,
            })
        })
        .collect()
}

/// 従業員の一覧から従業員を引く関数を作る（部分適用）。
pub fn employee_finder(employees: &[Employee]) -> impl Fn(&EmployeeId) -> Option<Employee> + '_ {
    move |id| employees.iter().find(|e| &e.id == id).cloned()
}

/// 企業の一覧から企業を引く関数を作る（部分適用）。
pub fn company_finder(companies: &[Company]) -> impl Fn(&CompanyId) -> Option<Company> + '_ {
    move |id| companies.iter().find(|c| &c.id == id).cloned()
}
