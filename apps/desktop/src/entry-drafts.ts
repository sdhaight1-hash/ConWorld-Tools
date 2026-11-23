// Only guard connected, unsaved or in-flight entry editors. No persistence,
// global publication state, or change to the Rust authority.
export function canLeaveEntryDrafts(): boolean {
  const draft = document.querySelector<HTMLElement>("[data-entry-draft-state]");
  if (!draft) return true;
  const notice = draft.querySelector<HTMLElement>("[data-draft-status]")!;
  notice.textContent = draft.dataset.entryDraftState === "saving"
    ? "Saving entry changes. Wait for the result before changing views."
    : "Unsaved page changes. Save or discard this draft before changing views.";
  notice.focus();
  return false;
}

// Browser unload protection is a supplementary guard; desktop native close
// behavior needs its own platform validation.
window.addEventListener("beforeunload", event => {
  if (!document.querySelector("[data-entry-draft-state]")) return;
  event.preventDefault();
  event.returnValue = "";
});
