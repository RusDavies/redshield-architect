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

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

type RedShieldWorkbenchHost = {
  tauriInvoke?: TauriInvoke;
};

declare global {
  interface Window {
    __RED_SHIELD_WORKBENCH__?: RedShieldWorkbenchHost;
    __TAURI__?: {
      core?: {
        invoke?: TauriInvoke;
      };
    };
  }
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

export class TauriLocalWorkbenchPersistence implements WorkbenchPersistence {
  constructor(private readonly invoke: TauriInvoke) {}

  async saveProposalDraft(key: string, draft: WorkbenchProposalDraft): Promise<void> {
    await this.invoke('redshield_save_proposal_draft', { key, draft });
  }

  async loadProposalDraft(key: string): Promise<ProposalDraftLoadResult> {
    const draft = await this.invoke<WorkbenchProposalDraft | null>('redshield_load_proposal_draft', {
      key,
    });
    return draft ? { status: 'found', draft } : { status: 'missing' };
  }

  async exportProposalDraft(draft: WorkbenchProposalDraft): Promise<void> {
    await this.invoke('redshield_export_proposal_draft', { draft });
  }
}

function resolveTauriInvoke(): TauriInvoke | undefined {
  return window.__RED_SHIELD_WORKBENCH__?.tauriInvoke ?? window.__TAURI__?.core?.invoke;
}

export function createWorkbenchPersistence(): WorkbenchPersistence {
  const tauriInvoke = resolveTauriInvoke();
  if (tauriInvoke) return new TauriLocalWorkbenchPersistence(tauriInvoke);
  return new BrowserLocalWorkbenchPersistence();
}

export const workbenchPersistence: WorkbenchPersistence = createWorkbenchPersistence();
