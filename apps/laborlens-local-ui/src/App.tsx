import { useEffect, useState } from "react";
import { DEMO_ATTENDANCE_REVIEW_RUN_ID, getAttendanceReviewSummary } from "./api";
import { modeCategories, workModes } from "./modeNavigation";
import type { ModeCategoryId, WorkModeId } from "./modeNavigation";
import { AttendanceReviewScreen } from "./screens/AttendanceReviewScreen";
import { ModePreviewScreen } from "./screens/ModePreviewScreen";
import type { AttendanceReviewSummaryResponse } from "./types";

export function App() {
  const [activeCategory, setActiveCategory] = useState<ModeCategoryId>("attendance");
  const [activeMode, setActiveMode] = useState<WorkModeId>("attendance");
  const [summary, setSummary] = useState<AttendanceReviewSummaryResponse | null>(null);

  const currentMode = workModes.find((mode) => mode.id === activeMode) ?? workModes[0];
  const categoryModes = workModes.filter((mode) => mode.category === activeCategory);

  useEffect(() => {
    void getAttendanceReviewSummary(DEMO_ATTENDANCE_REVIEW_RUN_ID)
      .then(setSummary)
      .catch(() => setSummary(null));
  }, []);

  function selectCategory(categoryId: ModeCategoryId) {
    setActiveCategory(categoryId);
    setActiveMode(workModes.find((mode) => mode.category === categoryId)?.id ?? "attendance");
  }

  return (
    <main className="shell">
      <header className="toolbar">
        <div>
          <h1>{currentMode.title}</h1>
          <p className="muted">{currentMode.summary}</p>
        </div>
      </header>

      <nav className="mode-nav" aria-label="メインカテゴリ">
        {modeCategories.map((category) => (
          <button
            className={category.id === activeCategory ? "is-active" : ""}
            key={category.id}
            type="button"
            onClick={() => selectCategory(category.id)}
          >
            {category.label}
          </button>
        ))}
      </nav>

      <nav className="submode-nav" aria-label="サブ画面">
        {categoryModes.map((mode) => (
          <button
            className={mode.id === activeMode ? "is-active" : ""}
            key={mode.id}
            type="button"
            onClick={() => setActiveMode(mode.id)}
          >
            {mode.label}
          </button>
        ))}
      </nav>

      {activeMode === "attendance" ? (
        <AttendanceReviewScreen />
      ) : (
        <ModePreviewScreen mode={currentMode} summary={summary} />
      )}
    </main>
  );
}
