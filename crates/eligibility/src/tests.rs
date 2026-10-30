//! Eligibility のテスト。データストアを使わず、固定のデータを返すクロージャを渡す（NFR7.2）。

use crate::*;

fn company(id: &str, status: ContractStatus) -> Company {
    Company {
        id: CompanyId::parse(id).unwrap(),
        name: format!("企業 {id}"),
        contract_status: status,
    }
}

fn employee(id: &str, company_id: &str) -> Employee {
    Employee {
        id: EmployeeId::parse(id).unwrap(),
        display_name: format!("従業員 {id}"),
        company_id: CompanyId::parse(company_id).unwrap(),
    }
}

fn fixtures() -> (Vec<Employee>, Vec<Company>) {
    (
        vec![employee("E001", "K001"), employee("E002", "K002")],
        vec![
            company("K001", ContractStatus::Active),
            company("K002", ContractStatus::Ended),
        ],
    )
}

#[test]
fn employee_of_active_company_is_eligible() {
    let (employees, companies) = fixtures();
    let id = EmployeeId::parse("E001").unwrap();
    let result = determine_eligibility(
        Some(&id),
        employee_finder(&employees),
        company_finder(&companies),
    );
    assert_eq!(result, EligibilityResult::Eligible);
}

#[test]
fn employee_of_ended_company_is_company_not_active() {
    let (employees, companies) = fixtures();
    let id = EmployeeId::parse("E002").unwrap();
    let result = determine_eligibility(
        Some(&id),
        employee_finder(&employees),
        company_finder(&companies),
    );
    assert_eq!(result, EligibilityResult::CompanyNotActive);
}

#[test]
fn unknown_or_missing_employee_is_employee_not_found() {
    // データストアの代わりに「誰も見つからない」関数を渡す
    let id = EmployeeId::parse("E999").unwrap();
    let nobody = |_: &EmployeeId| None;
    let no_company = |_: &CompanyId| None;
    assert_eq!(
        determine_eligibility(Some(&id), nobody, no_company),
        EligibilityResult::EmployeeNotFound
    );
    assert_eq!(
        determine_eligibility(None, nobody, no_company),
        EligibilityResult::EmployeeNotFound
    );
    assert!(EmployeeId::parse("E01").is_err());
    assert!(CompanyId::parse("K0001").is_err());
}

#[test]
fn switchable_employees_keep_seed_order_with_company_and_status() {
    let (employees, companies) = fixtures();
    let list = switchable_employees(&employees, &companies);
    let ids: Vec<&str> = list.iter().map(|e| e.employee_id.as_str()).collect();
    assert_eq!(ids, ["E001", "E002"]);
    assert_eq!(list[1].company_name, "企業 K002");
    assert_eq!(list[1].contract_status, ContractStatus::Ended);
}
