export const modeCategories = [
  { id: "attendance", label: "勤怠" },
  { id: "store", label: "店舗確認" },
  { id: "data", label: "データ確認" },
  { id: "analysis", label: "集計" },
  { id: "outputs", label: "出力" },
] as const;

export const workModes = [
  {
    id: "attendance",
    category: "attendance",
    label: "勤怠レビュー",
    title: "勤怠確認",
    summary: "給与計算の前に、打刻漏れや時刻の誤りを確認します。",
  },
  {
    id: "labor-risk",
    category: "attendance",
    label: "労働時間",
    title: "過重労働確認",
    summary: "長時間勤務、休日不足、休憩不足の候補を確認します。",
  },
  {
    id: "paid-leave",
    category: "attendance",
    label: "年次有給休暇",
    title: "年次有給休暇チェック",
    summary: "有休を取れているか、まだ少ない人がいないかを確認します。",
  },
  {
    id: "manager-load",
    category: "attendance",
    label: "店長負荷チェック",
    title: "店長負荷チェック",
    summary: "店長や責任者に勤務や作業が集まりすぎていないかをチェックします。",
  },
  {
    id: "store-corrections",
    category: "store",
    label: "店舗へ依頼",
    title: "店舗別修正依頼",
    summary: "店舗ごとに、修正が必要な勤怠データをまとめます。",
  },
  {
    id: "staffing",
    category: "store",
    label: "人員不足",
    title: "人手不足チェック",
    summary: "どの店舗の、どの曜日や時間帯で人員が不足しそうかをチェックします。",
  },
  {
    id: "master",
    category: "data",
    label: "従業員データ",
    title: "従業員データの不一致チェック",
    summary: "勤怠CSVの従業員IDをもとに、登録済みの氏名・部署・在籍状態と食い違いがないかをチェックします。",
  },
  {
    id: "csv-schema",
    category: "data",
    label: "CSV列チェック",
    title: "CSVの列チェック",
    summary: "CSVの列名変更や、必要な列の不足をチェックします。",
  },
  {
    id: "data-readiness",
    category: "data",
    label: "準備状況",
    title: "データの準備状況",
    summary: "どのデータが利用可能か、まだ足りないものは何かをチェックします。",
  },
  {
    id: "cost",
    category: "analysis",
    label: "人件費",
    title: "人件費チェック",
    summary: "人件費を店舗や部署ごとに見られるか、勤怠と合わせて確認します。",
  },
  {
    id: "group-analysis",
    category: "analysis",
    label: "集団分析",
    title: "集団集計",
    summary: "個人が分からない形で、部署や店舗ごとの傾向をチェックします。",
  },
  {
    id: "privacy",
    category: "outputs",
    label: "外部共有前",
    title: "共有前チェック",
    summary: "外に出してはいけない個人情報が入っていないかを確認します。",
  },
  {
    id: "report",
    category: "outputs",
    label: "月次レポート",
    title: "月次レポート",
    summary: "今月の勤怠、人件費、データ不備をまとめてチェックします。",
  },
  {
    id: "recheck",
    category: "outputs",
    label: "再確認",
    title: "修正後再確認",
    summary: "修正前と修正後を比べて、問題が減ったかを確認します。",
  },
] as const;

export type ModeCategoryId = (typeof modeCategories)[number]["id"];
export type WorkMode = (typeof workModes)[number];
export type WorkModeId = WorkMode["id"];
