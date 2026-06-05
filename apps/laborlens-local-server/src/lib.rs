use laborlens_rust::contexts::ingest::application::run_ingest_workflow;
use laborlens_rust::contexts::ingest::interfaces::{CsvInput, IngestRunCommand};
use laborlens_rust::shared::RunId;
use postgres::{Client, NoTls};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct LocalServer {
    demo_database: DemoDataStore,
}

impl Default for LocalServer {
    fn default() -> Self {
        let demo_database = env::var("LABORLENS_DEMO_DATABASE_URL")
            .ok()
            .and_then(|database_url| DemoDataStore::from_postgres(&database_url).ok())
            .filter(|database| database.employee_count() == 1_000)
            .unwrap_or_else(DemoDataStore::seeded);

        Self { demo_database }
    }
}

pub const USE_CASE_IDS: [&str; 14] = [
    "uc-01", "uc-02", "uc-03", "uc-04", "uc-05", "uc-06", "uc-07", "uc-08", "uc-09", "uc-10",
    "uc-11", "uc-12", "uc-13", "uc-14",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemoEmployee {
    pub employee_id: String,
    pub display_name: String,
    pub department: String,
    pub store_name: String,
    pub employment_type: String,
    pub role_name: String,
    pub hired_on: String,
}

#[derive(Debug, Clone)]
pub struct DemoDataStore {
    employees: Vec<DemoEmployee>,
}

impl DemoDataStore {
    pub fn seeded() -> Self {
        Self {
            employees: (1..=1_000).map(build_demo_employee).collect(),
        }
    }

    pub fn from_postgres(database_url: &str) -> Result<Self, postgres::Error> {
        let mut client = Client::connect(database_url, NoTls)?;
        let rows = client.query(
            "SELECT employee_id, display_name, department, store_name, \
                    employment_type, role_name, hired_on::text \
             FROM laborlens.demo_employees \
             WHERE seed_version = 'demo_japanese_employees.v1' \
             ORDER BY employee_id",
            &[],
        )?;

        Ok(Self {
            employees: rows
                .into_iter()
                .map(|row| DemoEmployee {
                    employee_id: row.get(0),
                    display_name: row.get(1),
                    department: row.get(2),
                    store_name: row.get(3),
                    employment_type: row.get(4),
                    role_name: row.get(5),
                    hired_on: row.get(6),
                })
                .collect(),
        })
    }

    pub fn employee_count(&self) -> usize {
        self.employees.len()
    }

    pub fn employees(&self) -> &[DemoEmployee] {
        &self.employees
    }

    fn sample_employees(&self, use_case_index: usize) -> Vec<&DemoEmployee> {
        if self.employees.is_empty() {
            return Vec::new();
        }

        let start = (use_case_index * 37) % self.employees.len();
        [
            start,
            (start + 113) % self.employees.len(),
            (start + 271) % self.employees.len(),
        ]
        .into_iter()
        .map(|index| &self.employees[index])
        .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCaseDefinition {
    pub use_case_id: String,
    pub button_label: String,
    pub title: String,
    pub actor: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemoDbSource {
    pub table_name: String,
    pub seed_version: String,
    pub employee_count: usize,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub unit: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCaseSampleRow {
    pub subject: String,
    pub group: String,
    pub primary_value: String,
    pub status: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCaseSampleResponse {
    pub use_case: UseCaseDefinition,
    pub source: DemoDbSource,
    pub metrics: Vec<MetricCard>,
    pub rows: Vec<UseCaseSampleRow>,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiResponse {
    pub status_code: u16,
    pub content_type: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct LocalServerRunRequest {
    pub run_id: RunId,
    pub employees_csv: CsvInput,
    pub attendance_csv: CsvInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactListing {
    pub artifact_name: String,
    pub stable_path: String,
    pub content_type: String,
}

impl ArtifactListing {
    pub fn run_summary(stable_path: impl Into<String>) -> Self {
        Self {
            artifact_name: "run_summary".to_string(),
            stable_path: stable_path.into(),
            content_type: "application/json".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalServerRunResponse {
    pub run_id: RunId,
    pub job_state: String,
    pub progress_percent: u8,
    pub artifacts: Vec<ArtifactListing>,
}

impl LocalServer {
    pub fn start_run(&self, request: LocalServerRunRequest) -> LocalServerRunResponse {
        let result = run_ingest_workflow(IngestRunCommand::new(
            request.run_id,
            request.employees_csv,
            request.attendance_csv,
        ));

        LocalServerRunResponse {
            run_id: result.run_id,
            job_state: result.job.current_state.as_str().to_string(),
            progress_percent: result.job.progress_percent,
            artifacts: artifact_list(),
        }
    }

    pub fn use_case_catalog(&self) -> Vec<UseCaseDefinition> {
        use_case_definitions()
    }

    pub fn use_case_sample(&self, use_case_id: &str) -> Option<UseCaseSampleResponse> {
        let use_case_index = USE_CASE_IDS
            .iter()
            .position(|candidate| *candidate == use_case_id)?;
        let definition = use_case_definitions()
            .into_iter()
            .find(|definition| definition.use_case_id == use_case_id)?;
        let employees = self.demo_database.sample_employees(use_case_index);
        let rows = build_use_case_rows(use_case_id, &employees);

        Some(UseCaseSampleResponse {
            use_case: definition,
            source: DemoDbSource {
                table_name: "laborlens.demo_employees".to_string(),
                seed_version: "demo_japanese_employees.v1".to_string(),
                employee_count: self.demo_database.employee_count(),
                note: "1000人分の架空日本人従業員 seed から読み込み".to_string(),
            },
            metrics: build_metrics(use_case_id, self.demo_database.employee_count()),
            rows,
            findings: build_findings(use_case_id),
            next_actions: build_next_actions(use_case_id),
        })
    }

    pub fn api_get(&self, path: &str) -> ApiResponse {
        match path.trim_end_matches('/') {
            "/api/use-cases" => json_response(200, &self.use_case_catalog()),
            endpoint if endpoint.starts_with("/api/use-cases/") => {
                let mut parts = endpoint.trim_start_matches("/api/use-cases/").split('/');
                match (parts.next(), parts.next(), parts.next()) {
                    (Some(use_case_id), Some("sample-data"), None) => {
                        match self.use_case_sample(use_case_id) {
                            Some(sample) => json_response(200, &sample),
                            None => json_response(
                                404,
                                &serde_json::json!({
                                    "error": "unknown_use_case",
                                    "use_case_id": use_case_id
                                }),
                            ),
                        }
                    }
                    _ => json_response(404, &serde_json::json!({ "error": "not_found" })),
                }
            }
            _ => json_response(404, &serde_json::json!({ "error": "not_found" })),
        }
    }
}

fn artifact_list() -> Vec<ArtifactListing> {
    vec![
        ArtifactListing::run_summary("run_summary.json"),
        ArtifactListing {
            artifact_name: "issues".to_string(),
            stable_path: "issues.csv".to_string(),
            content_type: "text/csv".to_string(),
        },
        ArtifactListing {
            artifact_name: "public_report_model".to_string(),
            stable_path: "public_report_model.json".to_string(),
            content_type: "application/json".to_string(),
        },
    ]
}

fn json_response<T: Serialize>(status_code: u16, value: &T) -> ApiResponse {
    ApiResponse {
        status_code,
        content_type: "application/json; charset=utf-8".to_string(),
        body: serde_json::to_string(value).expect("API response should serialize"),
    }
}

fn build_demo_employee(number: usize) -> DemoEmployee {
    const FAMILY_NAMES: [&str; 20] = [
        "佐藤",
        "鈴木",
        "高橋",
        "田中",
        "伊藤",
        "渡辺",
        "山本",
        "中村",
        "小林",
        "加藤",
        "吉田",
        "山田",
        "佐々木",
        "山口",
        "松本",
        "井上",
        "木村",
        "林",
        "清水",
        "斎藤",
    ];
    const GIVEN_NAMES: [&str; 20] = [
        "陽菜", "結衣", "葵", "凛", "美咲", "翔太", "蓮", "悠真", "大和", "湊", "直樹", "拓也",
        "真央", "彩", "優子", "健太", "誠", "舞", "亮", "恵",
    ];
    const DEPARTMENTS: [&str; 10] = [
        "営業部",
        "販売部",
        "人事部",
        "経理部",
        "物流部",
        "製造部",
        "情報システム部",
        "商品管理部",
        "カスタマー支援部",
        "企画部",
    ];
    const STORES: [&str; 12] = [
        "東京東店",
        "東京西店",
        "横浜店",
        "千葉店",
        "さいたま店",
        "名古屋店",
        "大阪北店",
        "大阪南店",
        "京都店",
        "神戸店",
        "福岡店",
        "札幌店",
    ];
    const EMPLOYMENT_TYPES: [&str; 4] = ["正社員", "契約社員", "パート", "アルバイト"];
    const ROLES: [&str; 5] = ["スタッフ", "主任", "店長", "事務担当", "部門責任者"];

    let index = number - 1;
    DemoEmployee {
        employee_id: format!("EMP-{number:04}"),
        display_name: format!(
            "{} {}",
            FAMILY_NAMES[index % FAMILY_NAMES.len()],
            GIVEN_NAMES[(index / FAMILY_NAMES.len()) % GIVEN_NAMES.len()]
        ),
        department: DEPARTMENTS[index % DEPARTMENTS.len()].to_string(),
        store_name: STORES[index % STORES.len()].to_string(),
        employment_type: EMPLOYMENT_TYPES[index % EMPLOYMENT_TYPES.len()].to_string(),
        role_name: ROLES[index % ROLES.len()].to_string(),
        hired_on: format!("20{:02}-{:02}-01", 14 + (index % 11), 1 + (index % 12)),
    }
}

fn use_case_definitions() -> Vec<UseCaseDefinition> {
    vec![
        use_case(
            "uc-01",
            "勤怠不備",
            "給与計算前に勤怠データの不備を確認したい",
            "事務員・労務担当者",
            "打刻漏れ、二重打刻、時刻逆転を給与計算前に整理する。",
        ),
        use_case(
            "uc-02",
            "店長負荷",
            "店長の労働時間が増えすぎている原因を整理したい",
            "店長・エリアマネージャー",
            "欠員対応や繁忙時間帯により負荷が集中していないか確認する。",
        ),
        use_case(
            "uc-03",
            "人件費配分",
            "部署ごとの人件費配分を確認したい",
            "経理担当者",
            "部署別、雇用区分別の人件費割合と結合可否を確認する。",
        ),
        use_case(
            "uc-04",
            "集団分析",
            "ストレスチェックの集団分析を安全に労務改善へつなげたい",
            "人事担当者",
            "個人値を出さず、部署単位の負荷傾向と少人数抑制を確認する。",
        ),
        use_case(
            "uc-05",
            "店舗差戻し",
            "本部が店舗から集めた CSV の不備を一覧で返したい",
            "本部管理担当者",
            "店舗別の不備件数と修正依頼チェックリストを作る。",
        ),
        use_case(
            "uc-06",
            "修正対象",
            "Excel で直す前にどこを直せばよいかだけ知りたい",
            "事務員・店舗担当者",
            "原本を変えず、修正対象の行、列、理由だけを表示する。",
        ),
        use_case(
            "uc-07",
            "整備状況",
            "経営者が分析を始める前にデータ整備の状況を知りたい",
            "経営者・事業責任者",
            "データセット別の ready / blocked と次の整備対象を要約する。",
        ),
        use_case(
            "uc-08",
            "仕様変更",
            "システム担当が CSV 仕様変更の影響を確認したい",
            "情報システム担当者",
            "旧形式と新形式のヘッダー差分、足りない項目、移行メモを確認する。",
        ),
        use_case(
            "uc-09",
            "マスタ照合",
            "入社・退職・異動後の従業員マスタ不一致を見つけたい",
            "人事・労務担当者",
            "従業員 ID、所属部署、在籍状態の不一致を優先表示する。",
        ),
        use_case(
            "uc-10",
            "労働時間",
            "長時間労働や残業上限の確認材料を作りたい",
            "労務担当者・店長",
            "法的判断ではなく、早めに確認すべき労働時間リスクを整理する。",
        ),
        use_case(
            "uc-11",
            "有給取得",
            "有給休暇の取得状況を確認したい",
            "人事・労務担当者",
            "有給取得の偏りと取得促進対象を部署別に確認する。",
        ),
        use_case(
            "uc-12",
            "人員不足",
            "採用や応援要請の判断材料を作りたい",
            "店長・エリアマネージャー",
            "曜日・時間帯別の不足傾向と採用、応援、配置見直しの材料を示す。",
        ),
        use_case(
            "uc-13",
            "月次レポート",
            "毎月の労務レポートを自動で作りたい",
            "本部管理担当者",
            "勤怠、人件費、CSV 不備を毎月同じ形式で横並びにする。",
        ),
        use_case(
            "uc-14",
            "外部共有前",
            "データを外部へ渡す前に個人情報が含まれていないか確認したい",
            "管理部門・システム担当者",
            "識別情報らしき列、推測リスク、マスキング対象を確認する。",
        ),
    ]
}

fn use_case(
    use_case_id: &str,
    button_label: &str,
    title: &str,
    actor: &str,
    summary: &str,
) -> UseCaseDefinition {
    UseCaseDefinition {
        use_case_id: use_case_id.to_string(),
        button_label: button_label.to_string(),
        title: title.to_string(),
        actor: actor.to_string(),
        summary: summary.to_string(),
    }
}

fn build_metrics(use_case_id: &str, employee_count: usize) -> Vec<MetricCard> {
    let scenario_number = use_case_number(use_case_id);
    vec![
        metric("seed 従業員数", employee_count.to_string(), "人", "ready"),
        metric(
            "対象データセット",
            dataset_label(use_case_id).to_string(),
            "",
            "ready",
        ),
        metric(
            "確認対象",
            (12 + scenario_number * 3).to_string(),
            "件",
            if scenario_number % 4 == 0 {
                "suppressed"
            } else {
                "attention"
            },
        ),
    ]
}

fn build_use_case_rows(use_case_id: &str, employees: &[&DemoEmployee]) -> Vec<UseCaseSampleRow> {
    let labels = row_labels(use_case_id);
    employees
        .iter()
        .enumerate()
        .map(|(index, employee)| UseCaseSampleRow {
            subject: if use_case_id == "uc-04" || use_case_id == "uc-07" {
                employee.department.clone()
            } else {
                format!("{} {}", employee.employee_id, employee.display_name)
            },
            group: format!("{} / {}", employee.store_name, employee.department),
            primary_value: labels[index % labels.len()].to_string(),
            status: row_status(use_case_id, index).to_string(),
            note: row_note(use_case_id, employee, index),
        })
        .collect()
}

fn build_findings(use_case_id: &str) -> Vec<String> {
    match use_case_id {
        "uc-01" => vec![
            "給与計算へ進める前に確認すべき勤怠 issue が残っています。",
            "未登録従業員と時刻逆転は優先度高として扱います。",
        ],
        "uc-02" => vec![
            "店長ロールの欠員対応が週末に集中しています。",
            "忙しい時間帯と勤務人数の不足が同じ期間に重なっています。",
        ],
        "uc-03" => vec![
            "人件費データは部署別月次と従業員別月次が混在しています。",
            "部署別配分は可能ですが、個人勤怠との直接結合には追加確認が必要です。",
        ],
        "uc-04" => vec![
            "少人数部署は集計結果を抑制しています。",
            "個人の疲労値やコメントは UI に出していません。",
        ],
        "uc-05" => vec![
            "店舗ごとに列名と日付形式の不一致があります。",
            "修正依頼は店舗単位で返せる状態です。",
        ],
        "uc-06" => vec![
            "原本 hash は変わっていません。",
            "修正対象の行、列、理由だけを抽出しています。",
        ],
        "uc-07" => vec![
            "勤怠と従業員マスタは ready、人件費と売上は partial です。",
            "分析開始前に追加で整えるべきデータが見えています。",
        ],
        "uc-08" => vec![
            "旧 CSV と新 CSV のヘッダー差分があります。",
            "移行期間中は fixture による再現確認が必要です。",
        ],
        "uc-09" => vec![
            "退職済み従業員と部署不一致が見つかっています。",
            "給与計算前にマスタ修正の確認が必要です。",
        ],
        "uc-10" => vec![
            "長時間労働の確認候補があります。",
            "適法・違法判断ではなく、労務担当者の確認材料として出力しています。",
        ],
        "uc-11" => vec![
            "有給取得が少ない部署が一部あります。",
            "個人を責める表示ではなく、取得しづらさの傾向として扱います。",
        ],
        "uc-12" => vec![
            "人員不足は特定曜日に偏っています。",
            "採用、応援、配置見直しの候補を分けて表示します。",
        ],
        "uc-13" => vec![
            "月次の勤怠、人件費、不備件数を同じ形式で表示できます。",
            "前月比較により改善と残課題を分けています。",
        ],
        "uc-14" => vec![
            "外部共有前に確認すべき識別情報らしき列があります。",
            "少人数部署や特殊勤務パターンは推測リスクとして扱います。",
        ],
        _ => vec![],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn build_next_actions(use_case_id: &str) -> Vec<String> {
    match use_case_id {
        "uc-01" => vec![
            "本人確認が必要な勤怠を店舗へ返す",
            "修正後 CSV で再実行する",
        ],
        "uc-02" => vec![
            "週末シフトの応援候補を確認する",
            "店長代替勤務の発生理由を記録する",
        ],
        "uc-03" => vec![
            "部署別月次データと従業員別月次データを分けて扱う",
            "個人勤怠と結合できない行を経理へ確認する",
        ],
        "uc-04" => vec![
            "少人数部署の集計を表示しない",
            "セルフケア案内文を人事担当者が確認する",
        ],
        "uc-05" => vec![
            "店舗別チェックリストを出力する",
            "前回提出分との差分を比較する",
        ],
        "uc-06" => vec![
            "Excel で対象セルだけ修正する",
            "raw input hash を再確認する",
        ],
        "uc-07" => vec![
            "blocked データセットの整備順を決める",
            "導入ステップをロードマップへ反映する",
        ],
        "uc-08" => vec![
            "新旧ヘッダー対応表を更新する",
            "移行 fixture で run を再実行する",
        ],
        "uc-09" => vec![
            "従業員マスタを人事へ確認する",
            "退職済み従業員の勤怠残存を調査する",
        ],
        "uc-10" => vec!["確認候補を労務担当者へ渡す", "会社の運用閾値を設定する"],
        "uc-11" => vec![
            "取得促進対象の部署を確認する",
            "管理者向け声かけ文面を確認する",
        ],
        "uc-12" => vec![
            "一時的欠員と慢性的不足を分ける",
            "応援要請または採用の検討材料にする",
        ],
        "uc-13" => vec![
            "月次レポートを成果物として保存する",
            "改善した点と残課題を分けて共有する",
        ],
        "uc-14" => vec![
            "不要な識別列をマスキングする",
            "外部共有可否を会社ルールで確認する",
        ],
        _ => vec![],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn metric(label: &str, value: String, unit: &str, status: &str) -> MetricCard {
    MetricCard {
        label: label.to_string(),
        value,
        unit: unit.to_string(),
        status: status.to_string(),
    }
}

fn dataset_label(use_case_id: &str) -> &'static str {
    match use_case_id {
        "uc-01" | "uc-02" | "uc-06" | "uc-10" => "勤怠",
        "uc-03" => "人件費",
        "uc-04" => "集団分析",
        "uc-05" => "店舗 CSV",
        "uc-07" => "準備状況",
        "uc-08" => "CSV 仕様",
        "uc-09" => "従業員マスタ",
        "uc-11" => "有給休暇",
        "uc-12" => "人員配置",
        "uc-13" => "月次レポート",
        "uc-14" => "外部共有前",
        _ => "デモ",
    }
}

fn row_labels(use_case_id: &str) -> &'static [&'static str] {
    match use_case_id {
        "uc-01" => &["打刻漏れ", "時刻逆転", "未登録従業員"],
        "uc-02" => &["欠員対応 18h", "週末連続勤務", "繁忙帯不足"],
        "uc-03" => &["部署別月次", "従業員別月次", "結合不可"],
        "uc-04" => &["集計抑制", "部署傾向のみ", "REDACTED"],
        "uc-05" => &["必須列不足", "日付形式エラー", "ID 表記揺れ"],
        "uc-06" => &["修正対象セル", "原本 hash 維持", "再確認待ち"],
        "uc-07" => &["ready", "partial", "blocked"],
        "uc-08" => &["旧ヘッダー", "新ヘッダー", "不足項目"],
        "uc-09" => &["部署不一致", "退職済み", "未登録"],
        "uc-10" => &["残業確認", "連続勤務", "休日取得確認"],
        "uc-11" => &["取得率低", "部署偏り", "声かけ候補"],
        "uc-12" => &["慢性不足", "一時欠員", "応援候補"],
        "uc-13" => &["前月改善", "残課題", "横並び比較"],
        "uc-14" => &["識別列候補", "推測リスク", "マスキング候補"],
        _ => &["確認対象"],
    }
}

fn row_status(use_case_id: &str, index: usize) -> &'static str {
    if use_case_id == "uc-04" && index == 0 {
        "suppressed"
    } else if index == 2 {
        "blocked"
    } else if index == 1 {
        "attention"
    } else {
        "ready"
    }
}

fn row_note(use_case_id: &str, employee: &DemoEmployee, index: usize) -> String {
    match use_case_id {
        "uc-04" => format!(
            "{} は個人値を表示せず、部署単位で扱います。",
            employee.department
        ),
        "uc-07" => format!(
            "{} のデータ整備状態を経営層向けに要約します。",
            employee.department
        ),
        "uc-14" => format!(
            "{} の外部共有前チェックで識別情報候補を確認します。",
            employee.store_name
        ),
        _ => format!(
            "{} の {} データからサンプル {} を読み込みました。",
            employee.store_name,
            employee.department,
            index + 1
        ),
    }
}

fn use_case_number(use_case_id: &str) -> usize {
    use_case_id
        .trim_start_matches("uc-")
        .parse::<usize>()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use crate::{ArtifactListing, DemoDataStore, LocalServer, LocalServerRunRequest, USE_CASE_IDS};
    use laborlens_rust::contexts::ingest::domain::DatasetKind;
    use laborlens_rust::contexts::ingest::infrastructure::load_csv_input_from_path;
    use laborlens_rust::shared::RunId;
    use std::path::PathBuf;

    fn fixture_path(relative_path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(relative_path)
    }

    #[test]
    fn start_run_returns_job_progress_and_artifact_listing() {
        let server = LocalServer::default();
        let request = LocalServerRunRequest {
            run_id: RunId::new("run-local-server-001"),
            employees_csv: load_csv_input_from_path(
                DatasetKind::Employees,
                fixture_path("fixtures/valid/ingest/employees.csv"),
            )
            .expect("employees fixture should load"),
            attendance_csv: load_csv_input_from_path(
                DatasetKind::Attendance,
                fixture_path("fixtures/valid/ingest/attendance.csv"),
            )
            .expect("attendance fixture should load"),
        };

        let response = server.start_run(request);

        assert_eq!(response.run_id.as_str(), "run-local-server-001");
        assert_eq!(response.job_state, "succeeded");
        assert_eq!(response.progress_percent, 100);
        assert!(response
            .artifacts
            .iter()
            .any(|artifact| artifact == &ArtifactListing::run_summary("run_summary.json")));
    }

    #[test]
    fn seeded_demo_database_contains_one_thousand_japanese_dummy_employees() {
        let database = DemoDataStore::seeded();

        assert_eq!(database.employee_count(), 1_000);
        assert_eq!(database.employees()[0].employee_id, "EMP-0001");
        assert_eq!(database.employees()[999].employee_id, "EMP-1000");
        assert!(database
            .employees()
            .iter()
            .all(|employee| employee.display_name.chars().any(|ch| !ch.is_ascii())));
    }

    #[test]
    fn use_case_catalog_exposes_all_documented_buttons() {
        let server = LocalServer::default();
        let catalog = server.use_case_catalog();

        assert_eq!(USE_CASE_IDS.len(), 14);
        assert_eq!(catalog.len(), USE_CASE_IDS.len());
        for use_case_id in USE_CASE_IDS {
            let definition = catalog
                .iter()
                .find(|definition| definition.use_case_id == use_case_id)
                .expect("documented use case should have a UI definition");
            assert!(!definition.button_label.is_empty());
            assert!(!definition.title.is_empty());
        }
    }

    #[test]
    fn every_use_case_button_loads_sample_data_from_seeded_database() {
        let server = LocalServer::default();

        for use_case_id in USE_CASE_IDS {
            let sample = server
                .use_case_sample(use_case_id)
                .expect("documented use case should load a sample");
            assert_eq!(sample.source.employee_count, 1_000);
            assert_eq!(sample.source.table_name, "laborlens.demo_employees");
            assert_eq!(sample.use_case.use_case_id, use_case_id);
            assert!(!sample.rows.is_empty());
            assert!(!sample.findings.is_empty());
            assert!(!sample.next_actions.is_empty());
        }
    }

    #[test]
    fn unknown_use_case_is_not_loaded() {
        let server = LocalServer::default();

        assert!(server.use_case_sample("uc-999").is_none());
    }
}
