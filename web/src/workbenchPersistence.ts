export type WorkbenchProposalDraft = {
  proposalId: string;
  schemaVersion: string;
  state: string;
  createdAt: string;
  intent: string;
  operations: unknown[];
};

export type ProposalDraftLoadResult =
  | { status: 'found'; draft: WorkbenchProposalDraft }
  | { status: 'missing' };

export interface WorkbenchPersistence {
  saveProposalDraft(key: string, draft: WorkbenchProposalDraft): Promise<void>;
  loadProposalDraft(key: string): Promise<ProposalDraftLoadResult>;
  exportProposalDraft(draft: WorkbenchProposalDraft): Promise<void>;
}

export class BrowserLocalWorkbenchPersistence implements WorkbenchPersistence {
  async saveProposalDraft(key: string, draft: WorkbenchProposalDraft): Promise<void> {
    window.localStorage.setItem(key, JSON.stringify(draft, null, 2));
  }

  async loadProposalDraft(key: string): Promise<ProposalDraftLoadResult> {
    const saved = window.localStorage.getItem(key);
    if (!saved) return { status: 'missing' };
    return {
      status: 'found',
      draft: JSON.parse(saved) as WorkbenchProposalDraft,
    };
  }

  async exportProposalDraft(draft: WorkbenchProposalDraft): Promise<void> {
    const blob = new Blob([`${JSON.stringify(draft, null, 2)}\n`], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${draft.proposalId}.${draft.state}.json`;
    anchor.click();
    URL.revokeObjectURL(url);
  }
}

export const workbenchPersistence: WorkbenchPersistence = new BrowserLocalWorkbenchPersistence();
