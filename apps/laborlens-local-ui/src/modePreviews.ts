import type { WorkModeId } from "./modeNavigation";
import type { AttendanceReviewSummaryResponse } from "./types";

export type PreviewMetric = {
  label: string;
  value: number;
  unit?: string;
  status?: string;
  helper?: string;
};

export type PreviewRow = {
  cells: string[];
  status: string;
};

export type PreviewConfig = {
  metrics: PreviewMetric[];
  filters: string[];
  tableTitle: string;
  columns: string[];
  rows: PreviewRow[];
  note: string;
};

export function getModePreview(
  modeId: WorkModeId,
  summary: AttendanceReviewSummaryResponse | null,
): PreviewConfig {
  const employees = summary?.metrics.find((metric) => metric.key === "employees")?.value ?? 1000;
  const issueRows = summary?.issue_rows ?? 100;
  const totalRows = summary?.total_rows ?? 1000;

  switch (modeId) {
    case "labor-risk":
      return preview(
        [
          metricPreview("確認候補", 38, "件", "attention", "残業・連続勤務・休憩不足"),
          metricPreview("対象者", 31, "人", "ready", "同一人物はまとめて表示"),
          metricPreview("優先確認", 12, "件", "blocked", "社内基準を超える可能性"),
        ],
        ["確認内容", "店舗", "部署", "期間", "優先度"],
        "過重労働確認リスト",
        ["確認内容", "対象", "期間", "理由", "確認者", "状態"],
        [
          previewRow(["長時間勤務", "東京東店 / 販売部", "2026-01", "残業が多い日が15件", "労務担当"], "attention"),
          previewRow(["休日不足", "大阪北店 / 店長", "第2週", "7日連続勤務の可能性", "エリアMG"], "attention"),
          previewRow(["休憩不足", "横浜店 / 物流部", "2026-01-18", "休憩45分未満の可能性", "店舗責任者"], "blocked"),
        ],
        "この画面は判断を決めるものではありません。担当者が確認するための材料です。",
      );
    case "paid-leave":
      return preview(
        [
          metricPreview("有休不足候補", 18, "人", "attention", "期限が近い人を含む"),
          metricPreview("対象部署", 6, "部署", "ready", "部署ごとに確認"),
          metricPreview("期限が近い人", 9, "人", "blocked", "90日以内に確認"),
        ],
        ["有休状況", "部署", "雇用区分", "年度", "年5日確認"],
        "年次有給休暇の確認リスト",
        ["部署", "雇用区分", "対象者", "有休5日未満", "期限", "状態"],
        [
          previewRow(["販売部", "正社員", "124人", "7人", "60日以内"], "attention"),
          previewRow(["物流部", "契約社員", "86人", "5人", "90日以内"], "attention"),
          previewRow(["カスタマー支援部", "パート", "142人", "6人", "対象者確認"], "ready"),
        ],
        "画面では正式には年次有給休暇、短く書くときは有休とします。",
      );
    case "manager-load":
      return preview(
        [
          metricPreview("負荷集中候補", 14, "件", "attention", "店長・責任者を確認"),
          metricPreview("対象店舗", 7, "店", "ready", "週末に多い"),
          metricPreview("欠員対応", 11, "回", "blocked", "代替勤務"),
        ],
        ["役割", "店舗", "曜日", "理由", "確認状況"],
        "店長負荷確認リスト",
        ["店舗", "役割", "曜日/時間帯", "理由", "次の確認", "状態"],
        [
          previewRow(["東京西店", "店長", "土曜 17-22時", "欠員対応", "応援候補を確認"], "attention"),
          previewRow(["名古屋店", "主任", "日曜 12-18時", "代替勤務", "休日変更の理由を確認"], "attention"),
          previewRow(["福岡店", "店長", "平日夜", "閉店作業が集中", "作業分担を確認"], "ready"),
        ],
        "人を評価する画面ではありません。仕事が一人に集まりすぎていないかをチェックする画面です。",
      );
    case "store-corrections":
      return preview(
        [
          metricPreview("依頼店舗", 8, "店", "attention", "確認が必要"),
          metricPreview("修正対象行", issueRows, "件", "blocked", "店舗確認待ち"),
          metricPreview("再提出待ち", 5, "店", "ready", "修正後CSV待ち"),
        ],
        ["店舗", "提出状況", "内容", "依頼状況", "担当"],
        "店舗別修正依頼リスト",
        ["店舗", "未確認", "主な内容", "依頼状況", "担当", "状態"],
        [
          previewRow(["東京東店", "18件", "出勤打刻漏れ", "差戻し準備", "店舗事務"], "attention"),
          previewRow(["横浜店", "12件", "退勤漏れ / 日付形式", "店舗確認中", "店長"], "attention"),
          previewRow(["大阪北店", "9件", "時刻逆転", "再提出待ち", "本部労務"], "ready"),
        ],
        "店舗に返す修正依頼CSVを、ここから確認できる想定です。",
      );
    case "staffing":
      return preview(
        [
          metricPreview("不足候補", 27, "枠", "attention", "曜日と時間で確認"),
          metricPreview("対象店舗", 9, "店", "ready", "売上と比べる"),
          metricPreview("慢性不足", 6, "枠", "blocked", "採用も検討"),
        ],
        ["店舗", "曜日", "時間帯", "不足状況", "売上データ"],
        "人員不足確認リスト",
        ["店舗", "曜日", "時間帯", "不足の内容", "理由", "状態"],
        [
          previewRow(["東京東店", "土曜", "17-21時", "3名不足候補", "売上ピークと勤務人数差"], "attention"),
          previewRow(["京都店", "日曜", "12-16時", "2名不足候補", "欠勤と来客増"], "attention"),
          previewRow(["札幌店", "平日", "19-22時", "慢性不足候補", "閉店作業の偏り"], "blocked"),
        ],
        "この画面だけで結論は出しません。売上、シフト、勤怠を比べて確認します。",
      );
    case "master":
      return preview(
        [
          metricPreview("不一致候補", 23, "件", "attention", "勤怠と従業員データ"),
          metricPreview("未登録候補", 7, "件", "blocked", "給与計算前に確認"),
          metricPreview("部署不一致候補", 11, "件", "ready", "異動日と比べる"),
        ],
        ["不一致種別", "部署", "雇用区分", "在籍状況", "期間"],
        "従業員データ不一致リスト",
        ["従業員ID", "氏名", "不一致種別", "勤怠データ", "従業員データ", "状態"],
        [
          previewRow(["EMP-0917", "佐藤 陽菜", "未登録", "東京東店 / 販売部", "該当なし"], "blocked"),
          previewRow(["EMP-0842", "田中 拓也", "退職済み", "2026-01勤怠あり", "2025-12退職"], "attention"),
          previewRow(["EMP-0761", "鈴木 舞", "部署不一致", "物流部", "商品管理部"], "attention"),
        ],
        "ここでは勤怠の問題ではなく、データ登録の誤りかどうかをチェックします。",
      );
    case "csv-schema":
      return preview(
        [
          metricPreview("影響あり", 16, "列", "attention", "知らない列や形式変更"),
          metricPreview("必須列不足", 3, "列", "blocked", "このままでは処理できない"),
          metricPreview("別名候補", 8, "列", "ready", "登録するか確認"),
        ],
        ["データの種類", "列の状況", "別名登録", "影響", "取り込み"],
        "CSVの列チェックリスト",
        ["データの種類", "CSVの列名", "見つかったこと", "困ること", "対応", "状態"],
        [
          previewRow(["勤怠", "退勤", "未登録別名", "退勤時刻に未対応", "別名辞書を確認"], "attention"),
          previewRow(["勤怠", "休憩", "必須列欠落", "勤務時間計算不可", "CSV出力設定を修正"], "blocked"),
          previewRow(["人件費", "部門コード", "未知列", "内部処理には未使用", "保持のみ"], "ready"),
        ],
        "列名を勝手に決めつけません。登録済みの別名だけを使います。",
      );
    case "data-readiness":
      return preview(
        [
          metricPreview("利用可能", 2, "件", "ready", "主な確認に使える"),
          metricPreview("一部利用可能", 2, "件", "attention", "できる集計が限られる"),
          metricPreview("利用不可", 1, "件", "blocked", "必要な入力が足りない"),
        ],
        ["データ", "準備状況", "細かさ", "優先度", "担当"],
        "データの準備リスト",
        ["データ", "状態", "不足内容", "利用可能画面", "次の確認", "確認"],
        [
          previewRow(["勤怠", "ready", "-", "勤怠レビュー / 労働時間", "月次更新"], "ready"),
          previewRow(["人件費", "partial", "従業員IDなし行", "人件費集計", "粒度確認"], "attention"),
          previewRow(["シフト", "blocked", "必須入力なし", "人員不足不可", "CSV追加"], "blocked"),
        ],
        "分析を始める前に、どのデータが使えるかをチェックします。",
      );
    case "cost":
      return preview(
        [
          metricPreview("結合可能", 2, "種類", "ready", "部署・雇用区分で集計"),
          metricPreview("結合不可", 6, "行", "blocked", "従業員IDがない"),
          metricPreview("要確認", 4, "件", "attention", "分け方を確認"),
        ],
        ["粒度", "部署", "店舗", "雇用区分", "結合可否"],
        "人件費集計リスト",
        ["対象", "粒度", "結合可否", "利用可能集計", "確認事項", "状態"],
        [
          previewRow(["部署別月次", "部署 x 月", "一部可", "部署別人件費", "個人の勤怠とは合わせない"], "ready"),
          previewRow(["従業員IDなし行", "月次合計", "不可", "全体集計のみ", "経理へ確認"], "blocked"),
          previewRow(["雇用区分別", "雇用区分 x 月", "一部可", "雇用区分別確認", "分け方を確認"], "attention"),
        ],
        "粒度が合わないデータは、個人の勤怠と無理に合わせません。",
      );
    case "group-analysis":
      return preview(
        [
          metricPreview("表示可能", 9, "集計", "ready", "個人が分からない形"),
          metricPreview("表示しない", 3, "部署", "suppressed", "5人未満"),
          metricPreview("確認する部署", 5, "部署", "attention", "部署の傾向だけ確認"),
        ],
        ["部署", "表示ルール", "集計粒度", "表示可否", "期間"],
        "部署別集計リスト",
        ["部署", "人数", "表示", "確認できる傾向", "表示しない情報", "状態"],
        [
          previewRow(["販売部", "124人", "公開可", "労働時間傾向", "個人疲労値"], "ready"),
          previewRow(["企画部", "4人", "抑制", "なし", "集計全体"], "suppressed"),
          previewRow(["物流部", "96人", "公開可", "勤務偏り", "個人コメント"], "attention"),
        ],
        "個人が分かる情報や、人数が少なすぎる部署の情報は表示しません。",
      );
    case "privacy":
      return preview(
        [
          metricPreview("確認対象", 9, "件", "attention", "共有前に確認"),
          metricPreview("少人数", 3, "部署", "suppressed", "表示しない"),
          metricPreview("個人情報候補", 2, "列", "blocked", "外部共有不可"),
        ],
        ["非表示理由", "共有先", "データ種別", "注意点", "共有可否"],
        "共有前確認リスト",
        ["対象", "注意点", "現在の扱い", "共有可否", "対応", "状態"],
        [
          previewRow(["自由記述列", "個人が分かる可能性", "除外", "不可", "共有対象から外す"], "blocked"),
          previewRow(["5人未満部署", "少人数で推測されやすい", "非表示", "不可", "集計を表示しない"], "suppressed"),
          previewRow(["公開用レポート", "名前は隠している", "確認待ち", "条件付きで可", "承認者に確認"], "attention"),
        ],
        "外へ出してよいかの最終判断は、会社のルールと担当者の確認で決めます。",
      );
    case "report":
      return preview(
        [
          metricPreview("今月の出力", 4, "種", "ready", "Markdown / CSV / JSON"),
          metricPreview("未確認", 2, "項目", "attention", "共有前に確認"),
          metricPreview("前月と比較", 1, "前月", "ready", "前月比"),
        ],
        ["対象月", "レポート種別", "出力状況", "再確認", "共有先"],
        "月次レポート一覧",
        ["出力物", "期間", "含める内容", "除外する内容", "共有状況", "状態"],
        [
          previewRow(["勤怠の問題まとめ", "2026-01", "問題の件数 / 店舗別", "個人の評価", "社内確認"], "ready"),
          previewRow(["人件費確認", "2026-01", "部署別集計", "給与確定の判断", "経理確認"], "attention"),
          previewRow(["再チェック結果", "実行結果の比較", "直った/残った/新しい問題", "元データの上書き", "管理者確認"], "ready"),
        ],
        "レポートは確認のための資料です。法律判断や人事評価の結論にはしません。",
      );
    case "recheck":
      return preview(
        [
          metricPreview("解消", 42, "件", "ready", "修正後に消えた問題"),
          metricPreview("残存", 11, "件", "attention", "もう一度確認"),
          metricPreview("新規", 3, "件", "blocked", "修正後に発生"),
        ],
        ["比較対象", "解消状況", "新規問題", "再提出", "店舗"],
        "修正前後比較",
        ["比較対象", "修正前", "修正後", "解消", "新規/残存", "状態"],
        [
          previewRow(["東京東店 勤怠", "18件", "4件", "14件", "残存4件"], "attention"),
          previewRow(["横浜店 勤怠", "12件", "0件", "12件", "新規0件"], "ready"),
          previewRow(["大阪北店 勤怠", "9件", "11件", "3件", "新規5件"], "blocked"),
        ],
        "修正後CSVは元のCSVとは別物として扱い、実行IDで比べます。",
      );
    default:
      return preview(
        [metricPreview("対象", employees, "人", "ready"), metricPreview("行数", totalRows, "行", "ready")],
        ["状態"],
        "確認リスト",
        ["対象", "内容", "状態"],
        [previewRow(["未設定", "この画面は今後作ります"], "ready")],
        "この画面は今後整えます。",
      );
  }
}

function preview(
  metrics: PreviewMetric[],
  filters: string[],
  tableTitle: string,
  columns: string[],
  rows: PreviewRow[],
  note: string,
): PreviewConfig {
  return {
    metrics,
    filters,
    tableTitle,
    columns,
    rows,
    note,
  };
}

function metricPreview(
  label: string,
  value: number,
  unit?: string,
  status?: string,
  helper?: string,
): PreviewMetric {
  return { label, value, unit, status, helper };
}

function previewRow(cells: string[], status: string): PreviewRow {
  return { cells, status };
}
