import { useCallback, useMemo, useRef, useState, type CSSProperties } from 'react';
import {
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  Background,
  Controls,
  Handle,
  MiniMap,
  Position,
  ReactFlow,
  type Connection,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
  type NodeProps,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import ELK from 'elkjs/lib/elk.bundled.js';

import manifest from '../../examples/minimal/redshield/manifest.json';
import elementsFile from '../../examples/minimal/redshield/model/elements.json';
import relationshipsFile from '../../examples/minimal/redshield/model/relationships.json';
import diagramsFile from '../../examples/minimal/redshield/views/diagrams.json';
import renderProfileFile from '../../examples/minimal/redshield/views/render-profile.json';
import traceFile from '../../examples/minimal/redshield/trace/links.json';

type ElementRecord = (typeof elementsFile.elements)[number];
type RelationshipRecord = (typeof relationshipsFile.relationships)[number];
type DiagramLayout = NonNullable<(typeof diagramsFile.diagrams)[number]['layout']>;
type DiagramNodeLayout = DiagramLayout['nodes'][number];
type DiagramConnectorLayout = DiagramLayout['connectors'][number];
type PortSide = 'top' | 'right' | 'bottom' | 'left';
type RendererId =
  | 'uml.actor'
  | 'uml.use_case'
  | 'uml.class'
  | 'uml.component'
  | 'uml.activity'
  | 'uml.sequence_participant'
  | 'image.element'
  | 'html.custom';
type RenderStyle = {
  fillColor?: string;
  strokeColor?: string;
  textColor?: string;
  lineStyle?: 'solid' | 'dashed' | 'dotted';
};
type RenderPort = {
  id: string;
  side: PortSide;
  offset?: number;
};
type RenderTarget = {
  rendererId: RendererId;
  assetRef?: string;
  style?: RenderStyle;
  ports?: RenderPort[];
  label?: {
    visible?: boolean;
    position?: 'inside' | 'top' | 'right' | 'bottom' | 'left';
  };
};
type RenderSelector = {
  elementId?: string;
  elementKind?: string;
  stereotype?: string;
  tag?: string;
};
type RenderRule = {
  id: string;
  description?: string;
  selector: RenderSelector;
  renderAs: RenderTarget;
  precedence: number;
  enabled?: boolean;
};
type RenderAsset = {
  id: string;
  uri: string;
  kind: 'image/png' | 'image/jpeg' | 'image/svg+xml';
  status: 'referenced' | 'available' | 'missing' | 'blocked';
  alt?: string;
};
type RenderProfile = {
  id: string;
  title: string;
  rules: RenderRule[];
  fallback: RenderTarget;
  assets?: RenderAsset[];
};
type RedshieldNodeData = {
  label: string;
  modelId: string;
  kind: string;
  description: string;
  stereotypes: string[];
  tags: string[];
  layoutState: 'generated' | 'manual';
  bounds: { width: number; height: number };
  labelPosition?: { x: number; y: number };
  render: RenderTarget;
  matchedRuleId: string;
  asset?: RenderAsset;
};
type RedshieldEdgeData = {
  relationshipId: string;
  relationshipKind: string;
  label: string;
  traceCount: number;
  layoutState: 'generated' | 'manual' | 'draft';
  routeHint: 'straight' | 'step' | 'smoothstep' | 'bezier' | 'orthogonal';
  labelPosition?: { x: number; y: number };
};
type ProposalOperation = {
  opId: string;
  op: string;
  args: Record<string, unknown>;
  rationale: string;
  sourceRefs?: string[];
};
type ProposalOperationDraft = Omit<ProposalOperation, 'opId'>;
type ProposalState = 'draft' | 'accepted';

const elk = new ELK();
const diagram = diagramsFile.diagrams[0];
const renderProfile = renderProfileFile.profiles[0] as RenderProfile;
const proposalStorageKey = `redshield.workbench.${diagram.id}.proposalDraft`;
const elementById = new Map(elementsFile.elements.map((element) => [element.id, element]));
const renderAssetById = new Map((renderProfile.assets ?? []).map((asset) => [asset.id, asset]));
const nodeLayoutByRef = new Map(
  (diagram.layout?.nodes ?? []).map((nodeLayout) => [nodeLayout.modelRef, nodeLayout]),
);
const connectorLayoutByRef = new Map(
  (diagram.layout?.connectors ?? []).map((connectorLayout) => [
    connectorLayout.relationshipRef,
    connectorLayout,
  ]),
);

function initialNodes(): Node<RedshieldNodeData>[] {
  return diagram.modelRefs
    .map((modelId, index): Node<RedshieldNodeData> | undefined => {
      const element = elementById.get(modelId);
      if (!element) return undefined;
      const persisted = nodeLayoutByRef.get(modelId);
      return {
        id: element.id,
        type: 'redshield',
        position: persisted
          ? { x: persisted.bounds.x, y: persisted.bounds.y }
          : { x: element.kind === 'actor' ? 80 : 360, y: 92 + index * 116 },
        data: toNodeData(element, persisted),
      };
    })
    .filter((node): node is Node<RedshieldNodeData> => node !== undefined);
}

function initialEdges(): Edge<RedshieldEdgeData>[] {
  return relationshipsFile.relationships
    .filter(
      (relationship) =>
        diagram.modelRefs.includes(relationship.sourceId) &&
        diagram.modelRefs.includes(relationship.targetId),
    )
    .map((relationship) => ({
      id: relationship.id,
      source: relationship.sourceId,
      target: relationship.targetId,
      label: relationship.label,
      type: toReactFlowEdgeType(
        toRouteHint(connectorLayoutByRef.get(relationship.id)?.routeHint?.kind),
      ),
      data: toEdgeData(relationship, connectorLayoutByRef.get(relationship.id)),
    }));
}

function toNodeData(element: ElementRecord, layout?: DiagramNodeLayout): RedshieldNodeData {
  const resolution = resolveRenderTarget(element);
  return {
    label: element.name,
    modelId: element.id,
    kind: element.kind,
    description: element.description,
    stereotypes: element.stereotypes ?? [],
    tags: element.tags,
    layoutState: toLayoutState(layout?.layoutState),
    bounds: {
      width: layout?.bounds.width ?? 210,
      height: layout?.bounds.height ?? 86,
    },
    labelPosition: layout?.labelPosition,
    render: resolution.render,
    matchedRuleId: resolution.ruleId,
    asset: resolution.render.assetRef ? renderAssetById.get(resolution.render.assetRef) : undefined,
  };
}

function resolveRenderTarget(element: ElementRecord): { render: RenderTarget; ruleId: string } {
  const rule = renderProfile.rules
    .filter((candidate) => candidate.enabled !== false && matchesSelector(element, candidate))
    .sort((left, right) => right.precedence - left.precedence)[0];

  return rule
    ? { render: rule.renderAs, ruleId: rule.id }
    : { render: renderProfile.fallback, ruleId: 'fallback' };
}

function matchesSelector(element: ElementRecord, rule: RenderRule) {
  const selector = rule.selector;
  if ('elementId' in selector && selector.elementId !== element.id) return false;
  if ('elementKind' in selector && selector.elementKind !== element.kind) return false;
  if (selector.stereotype && !(element.stereotypes ?? []).includes(selector.stereotype)) {
    return false;
  }
  if (selector.tag && !element.tags.includes(selector.tag)) return false;
  return true;
}

function toEdgeData(
  relationship: RelationshipRecord,
  layout?: DiagramConnectorLayout,
): RedshieldEdgeData {
  return {
    relationshipId: relationship.id,
    relationshipKind: relationship.relationshipKind,
    label: relationship.label,
    traceCount: traceFile.links.filter((link) => link.targetId === relationship.targetId).length,
    layoutState: toEdgeLayoutState(layout?.layoutState),
    routeHint: toRouteHint(layout?.routeHint?.kind),
    labelPosition: layout?.labelPosition,
  };
}

function toReactFlowEdgeType(routeKind?: RedshieldEdgeData['routeHint']) {
  if (routeKind === 'straight') return 'straight';
  if (routeKind === 'step' || routeKind === 'orthogonal') return 'step';
  if (routeKind === 'bezier') return 'default';
  return 'smoothstep';
}

function toLayoutState(value?: string): RedshieldNodeData['layoutState'] {
  return value === 'manual' ? 'manual' : 'generated';
}

function toEdgeLayoutState(value?: string): RedshieldEdgeData['layoutState'] {
  return value === 'manual' ? 'manual' : 'generated';
}

function toRouteHint(value?: string): RedshieldEdgeData['routeHint'] {
  if (
    value === 'straight' ||
    value === 'step' ||
    value === 'smoothstep' ||
    value === 'bezier' ||
    value === 'orthogonal'
  ) {
    return value;
  }
  return 'smoothstep';
}

function RedshieldNode({ data, selected }: NodeProps<Node<RedshieldNodeData>>) {
  const style = {
    '--node-fill': data.render.style?.fillColor,
    '--node-stroke': data.render.style?.strokeColor,
    '--node-text': data.render.style?.textColor,
    width: data.bounds.width,
    minHeight: data.bounds.height,
  } as CSSProperties;
  const className = [
    'diagram-node',
    `diagram-node--${data.kind}`,
    `diagram-node--renderer-${data.render.rendererId.replace('.', '-')}`,
    selected ? 'is-selected' : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={className} style={style}>
      <RenderHandles render={data.render} />
      <NodeRenderer data={data} />
    </div>
  );
}

function RenderHandles({ render }: { render: RenderTarget }) {
  const ports =
    render.ports && render.ports.length > 0
      ? render.ports
      : [
          { id: 'in', side: 'left' as const, offset: 0.5 },
          { id: 'out', side: 'right' as const, offset: 0.5 },
        ];

  return (
    <>
      {ports.map((port) => (
        <Handle
          key={port.id}
          id={port.id}
          type={port.side === 'left' || port.side === 'top' ? 'target' : 'source'}
          position={toHandlePosition(port.side)}
          style={toHandleStyle(port.side, port.offset ?? 0.5)}
        />
      ))}
    </>
  );
}

function toHandlePosition(side: 'top' | 'right' | 'bottom' | 'left') {
  if (side === 'top') return Position.Top;
  if (side === 'right') return Position.Right;
  if (side === 'bottom') return Position.Bottom;
  return Position.Left;
}

function toHandleStyle(side: 'top' | 'right' | 'bottom' | 'left', offset: number) {
  const percent = `${Math.max(0, Math.min(1, offset)) * 100}%`;
  if (side === 'top' || side === 'bottom') return { left: percent };
  return { top: percent };
}

function NodeRenderer({ data }: { data: RedshieldNodeData }) {
  if (data.render.rendererId === 'uml.actor') return <ActorNode data={data} />;
  if (data.render.rendererId === 'uml.component') return <ComponentNode data={data} />;
  if (data.render.rendererId === 'image.element') return <ImageNode data={data} />;
  return <ClassLikeNode data={data} />;
}

function ActorNode({ data }: { data: RedshieldNodeData }) {
  return (
    <div className="actor-renderer">
      <div className="actor-renderer__glyph" aria-hidden="true">
        <span className="actor-renderer__head" />
        <span className="actor-renderer__body" />
        <span className="actor-renderer__arms" />
        <span className="actor-renderer__legs" />
      </div>
      <NodeLabel data={data} />
    </div>
  );
}

function ComponentNode({ data }: { data: RedshieldNodeData }) {
  return (
    <>
      <div className="component-renderer__tabs" aria-hidden="true">
        <span />
        <span />
      </div>
      <NodeLabel data={data} />
    </>
  );
}

function ImageNode({ data }: { data: RedshieldNodeData }) {
  const isAvailable = data.asset?.status === 'available';
  return (
    <div className="image-renderer">
      <div className="image-renderer__asset" aria-label={data.asset?.alt ?? data.label}>
        {isAvailable ? (
          <img src={`/${data.asset?.uri}`} alt={data.asset?.alt ?? data.label} />
        ) : (
          <div className="image-renderer__placeholder">
            <span>{data.asset?.kind ?? 'image'}</span>
            <strong>{data.asset?.status ?? 'missing'}</strong>
          </div>
        )}
      </div>
      <NodeLabel data={data} />
    </div>
  );
}

function ClassLikeNode({ data }: { data: RedshieldNodeData }) {
  return (
    <>
      <NodeLabel data={data} />
      <div className="class-renderer__compartment">attributes</div>
      <div className="class-renderer__compartment">operations</div>
    </>
  );
}

function NodeLabel({ data }: { data: RedshieldNodeData }) {
  return (
    <div className="diagram-node__content">
      <div className="diagram-node__kind">{data.kind.replace('_', ' ')}</div>
      {data.stereotypes.length > 0 ? (
        <div className="diagram-node__stereotype">&lt;&lt;{data.stereotypes.join(', ')}&gt;&gt;</div>
      ) : null}
      <div className="diagram-node__label">{data.label}</div>
      <div className="diagram-node__id">{data.modelId}</div>
    </div>
  );
}

const nodeTypes = { redshield: RedshieldNode };

export default function App() {
  const [nodes, setNodes] = useState<Node<RedshieldNodeData>[]>(() => initialNodes());
  const [edges, setEdges] = useState<Edge<RedshieldEdgeData>[]>(() => initialEdges());
  const [selection, setSelection] = useState<{
    nodes: Node<RedshieldNodeData>[];
    edges: Edge<RedshieldEdgeData>[];
  }>({ nodes: [], edges: [] });
  const operationSequence = useRef(1);
  const proposalCreatedAt = useRef(new Date().toISOString());
  const [operationLog, setOperationLog] = useState<ProposalOperation[]>([]);
  const [proposalState, setProposalState] = useState<ProposalState>('draft');
  const [proposalStatus, setProposalStatus] = useState('No saved proposal draft.');

  const selectedNodeIds = useMemo(
    () => new Set(selection.nodes.map((node) => node.id)),
    [selection.nodes],
  );

  const recordOperations = useCallback((drafts: ProposalOperationDraft[]) => {
    if (drafts.length === 0) return;
    setProposalState('draft');
    setOperationLog((current) => {
      const operations = drafts.map((draft) => ({
        ...draft,
        opId: `op.view.${operationSequence.current++}`,
      }));
      return [...current, ...operations].slice(-12);
    });
  }, []);

  const recordOperation = useCallback(
    (draft: ProposalOperationDraft) => recordOperations([draft]),
    [recordOperations],
  );

  const onNodesChange = useCallback(
    (changes: NodeChange<Node<RedshieldNodeData>>[]) => {
      const moveOperations = changes
        .filter(
          (
            change,
          ): change is NodeChange<Node<RedshieldNodeData>> & {
            id: string;
            position: { x: number; y: number };
          } =>
            change.type === 'position' &&
            change.dragging === false &&
            'position' in change &&
            change.position !== undefined,
        )
        .map((change) => ({
          op: 'move_diagram_node',
          args: {
            diagramId: diagram.id,
            modelRef: change.id,
            x: Math.round(change.position.x),
            y: Math.round(change.position.y),
          },
          rationale: 'Node was moved on the workbench canvas.',
          sourceRefs: ['workbench.canvas'],
        }));

      setNodes((currentNodes) => {
        const changedIds = new Set(moveOperations.map((operation) => operation.args.modelRef));
        return applyNodeChanges(changes, currentNodes).map((node) =>
          changedIds.has(node.id)
            ? { ...node, data: { ...node.data, layoutState: 'manual' } }
            : node,
        );
      });
      recordOperations(moveOperations);
    },
    [recordOperations],
  );

  const onEdgesChange = useCallback((changes: EdgeChange<Edge<RedshieldEdgeData>>[]) => {
    setEdges((currentEdges) => applyEdgeChanges(changes, currentEdges));
  }, []);

  const onConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      const id = `rel.draft.${connection.source}.${connection.target}`;
      setEdges((currentEdges) =>
        addEdge(
          {
            ...connection,
            id,
            type: 'smoothstep',
            label: 'draft',
            data: {
              relationshipId: id,
              relationshipKind: 'association',
              label: 'draft',
              traceCount: 0,
              layoutState: 'draft',
              routeHint: 'smoothstep',
            },
          },
          currentEdges,
        ),
      );
      recordOperations([
        {
          op: 'create_relationship',
          args: {
            id,
            relationshipKind: 'association',
            sourceId: connection.source,
            targetId: connection.target,
            label: 'draft',
          },
          rationale: 'Canvas connector created a draft semantic relationship.',
          sourceRefs: ['workbench.canvas'],
        },
        {
          op: 'connect_diagram_relationship',
          args: {
            diagramId: diagram.id,
            relationshipRef: id,
            routeHint: {
              kind: 'smoothstep',
            },
          },
          rationale: 'Canvas connector made the draft relationship visible in this diagram view.',
          sourceRefs: ['workbench.canvas'],
        },
      ]);
    },
    [recordOperations],
  );

  const runElkLayout = useCallback(async () => {
    const graph = {
      id: 'redshield-use-case',
      layoutOptions: {
        'elk.algorithm': 'layered',
        'elk.direction': 'RIGHT',
        'elk.spacing.nodeNode': '60',
        'elk.layered.spacing.nodeNodeBetweenLayers': '90',
      },
      children: nodes.map((node) => ({
        id: node.id,
        width: node.data.bounds.width,
        height: node.data.bounds.height,
      })),
      edges: edges.map((edge) => ({
        id: edge.id,
        sources: [edge.source],
        targets: [edge.target],
      })),
    };

    const layouted = await elk.layout(graph);
    const positions = new Map<string, { x: number; y: number }>(
      layouted.children?.map((child: { id: string; x?: number; y?: number }) => [
        child.id,
        { x: child.x ?? 0, y: child.y ?? 0 },
      ]) ?? [],
    );
    setNodes((currentNodes) =>
      currentNodes.map((node) => ({
        ...node,
        position: positions.get(node.id) ?? node.position,
        data: { ...node.data, layoutState: 'generated' },
      })),
    );
    recordOperation({
      op: 'apply_diagram_auto_layout',
      args: {
        diagramId: diagram.id,
        layoutEngine: 'elk.layered',
        nodes: nodes.map((node) => {
          const position = positions.get(node.id) ?? node.position;
          return {
            modelRef: node.id,
            bounds: {
              x: Math.round(position.x),
              y: Math.round(position.y),
              width: node.data.bounds.width,
              height: node.data.bounds.height,
            },
            layoutState: 'generated',
            labelPosition: node.data.labelPosition,
          };
        }),
        connectors: edges.map((edge) => ({
          relationshipRef: edge.data?.relationshipId ?? edge.id,
          layoutState: 'generated',
          routeHint: {
            kind: edge.data?.routeHint ?? 'smoothstep',
          },
          labelPosition: edge.data?.labelPosition,
        })),
      },
      rationale: 'ELK auto-layout generated canvas bounds for this diagram view.',
      sourceRefs: ['workbench.elk'],
    });
  }, [edges, nodes, recordOperation]);

  const alignSelected = useCallback(
    (mode: 'left' | 'right' | 'top' | 'bottom' | 'hcenter' | 'vcenter') => {
      if (selectedNodeIds.size < 2) return;
      const selectedNodes = nodes.filter((node) => selectedNodeIds.has(node.id));
      const minX = Math.min(...selectedNodes.map((node) => node.position.x));
      const maxX = Math.max(...selectedNodes.map((node) => node.position.x));
      const minY = Math.min(...selectedNodes.map((node) => node.position.y));
      const maxY = Math.max(...selectedNodes.map((node) => node.position.y));
      const centerX = (minX + maxX) / 2;
      const centerY = (minY + maxY) / 2;

      setNodes((currentNodes) =>
        currentNodes.map((node) => {
          if (!selectedNodeIds.has(node.id)) return node;
          const next = { ...node.position };
          if (mode === 'left') next.x = minX;
          if (mode === 'right') next.x = maxX;
          if (mode === 'top') next.y = minY;
          if (mode === 'bottom') next.y = maxY;
          if (mode === 'hcenter') next.x = centerX;
          if (mode === 'vcenter') next.y = centerY;
          return { ...node, position: next, data: { ...node.data, layoutState: 'manual' } };
        }),
      );
      recordOperation({
        op: 'align_diagram_nodes',
        args: {
          diagramId: diagram.id,
          modelRefs: Array.from(selectedNodeIds),
          alignment: mode,
        },
        rationale: 'Selected nodes were aligned on the workbench canvas.',
        sourceRefs: ['workbench.canvas'],
      });
    },
    [nodes, recordOperation, selectedNodeIds],
  );

  const distributeSelected = useCallback(
    (axis: 'x' | 'y') => {
      if (selectedNodeIds.size < 3) return;
      const selectedNodes = nodes
        .filter((node) => selectedNodeIds.has(node.id))
        .sort((left, right) =>
          axis === 'x' ? left.position.x - right.position.x : left.position.y - right.position.y,
        );
      const first = selectedNodes[0].position[axis];
      const last = selectedNodes[selectedNodes.length - 1].position[axis];
      const step = (last - first) / (selectedNodes.length - 1);
      const positions = new Map(
        selectedNodes.map((node, index) => [node.id, first + index * step]),
      );

      setNodes((currentNodes) =>
        currentNodes.map((node) => {
          const value = positions.get(node.id);
          if (value === undefined) return node;
          return {
            ...node,
            position: { ...node.position, [axis]: value },
            data: { ...node.data, layoutState: 'manual' },
          };
        }),
      );
      recordOperation({
        op: 'distribute_diagram_nodes',
        args: {
          diagramId: diagram.id,
          modelRefs: Array.from(selectedNodeIds),
          axis,
        },
        rationale: 'Selected nodes were distributed on the workbench canvas.',
        sourceRefs: ['workbench.canvas'],
      });
    },
    [nodes, recordOperation, selectedNodeIds],
  );

  const viewMetadata = useMemo(
    () => ({
      diagramId: diagram.id,
      coordinateSystem: 'canvas',
      layoutEngine: diagram.layout?.layoutEngine ?? 'manual-or-elk',
      layoutState: nodes.some((node) => node.data.layoutState === 'manual') ? 'mixed' : 'generated',
      nodes: nodes.map((node) => ({
        modelRef: node.id,
        bounds: {
          x: Math.round(node.position.x),
          y: Math.round(node.position.y),
          width: node.data.bounds.width,
          height: node.data.bounds.height,
        },
        layoutState: node.data.layoutState,
        labelPosition: node.data.labelPosition,
      })),
      connectors: edges.map((edge) => ({
        relationshipRef: edge.data?.relationshipId ?? edge.id,
        layoutState: edge.data?.layoutState ?? 'manual',
        routeHint: {
          kind: edge.data?.routeHint ?? 'smoothstep',
        },
        labelPosition: edge.data?.labelPosition,
      })),
    }),
    [edges, nodes],
  );
  const proposalDraft = useMemo(
    () => ({
      proposalId: 'proposal.workbench-draft',
      schemaVersion: '0.1.0',
      state: proposalState,
      createdAt: proposalCreatedAt.current,
      intent: 'Apply direct manipulation changes from the workbench canvas.',
      operations: operationLog,
    }),
    [operationLog, proposalState],
  );
  const saveProposalDraft = useCallback(() => {
    window.localStorage.setItem(proposalStorageKey, JSON.stringify(proposalDraft, null, 2));
    setProposalStatus(`Saved ${proposalDraft.operations.length} operations locally.`);
  }, [proposalDraft]);
  const loadProposalDraft = useCallback(() => {
    const saved = window.localStorage.getItem(proposalStorageKey);
    if (!saved) {
      setProposalStatus('No local proposal draft to load.');
      return;
    }
    const parsed = JSON.parse(saved) as {
      state?: string;
      operations?: ProposalOperation[];
    };
    const operations = Array.isArray(parsed.operations) ? parsed.operations : [];
    setOperationLog(operations);
    setProposalState(parsed.state === 'accepted' ? 'accepted' : 'draft');
    const maxSequence = operations.reduce((max, operation) => {
      const sequence = Number(operation.opId.replace('op.view.', ''));
      return Number.isFinite(sequence) ? Math.max(max, sequence) : max;
    }, 0);
    operationSequence.current = Math.max(operationSequence.current, maxSequence + 1);
    setProposalStatus(`Loaded ${operations.length} operations from local storage.`);
  }, []);
  const downloadProposalDraft = useCallback(() => {
    const blob = new Blob([`${JSON.stringify(proposalDraft, null, 2)}\n`], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${proposalDraft.proposalId}.${proposalDraft.state}.json`;
    anchor.click();
    URL.revokeObjectURL(url);
    setProposalStatus(`Downloaded ${proposalDraft.state} proposal JSON.`);
  }, [proposalDraft]);
  const clearProposalDraft = useCallback(() => {
    setOperationLog([]);
    setProposalState('draft');
    setProposalStatus('Cleared in-memory proposal operations.');
  }, []);

  return (
    <main className="workbench-shell">
      <aside className="sidebar">
        <div className="brand">
          <span>RedShield Architect</span>
          <strong>{manifest.name}</strong>
        </div>
        <section>
          <h2>Model</h2>
          <div className="object-list">
            {elementsFile.elements.map((element) => (
              <button
                key={element.id}
                className={selectedNodeIds.has(element.id) ? 'is-active' : ''}
                onClick={() =>
                  setSelection({
                    nodes: nodes.filter((node) => node.id === element.id),
                    edges: [],
                  })
                }
              >
                <span>{element.name}</span>
                <small>{element.kind.replace('_', ' ')}</small>
              </button>
            ))}
          </div>
        </section>
        <section>
          <h2>Trace</h2>
          <div className="trace-list">
            {traceFile.links.map((link) => (
              <div key={link.id}>
                <strong>{link.traceKind}</strong>
                <span>{link.sourceId}</span>
                <span>{link.targetId}</span>
              </div>
            ))}
          </div>
        </section>
      </aside>

      <section className="canvas-region">
        <div className="toolbar">
          <button onClick={runElkLayout}>Auto layout</button>
          <span className="divider" />
          <button disabled={selection.nodes.length < 2} onClick={() => alignSelected('left')}>
            Align L
          </button>
          <button disabled={selection.nodes.length < 2} onClick={() => alignSelected('hcenter')}>
            Align X
          </button>
          <button disabled={selection.nodes.length < 2} onClick={() => alignSelected('right')}>
            Align R
          </button>
          <button disabled={selection.nodes.length < 2} onClick={() => alignSelected('top')}>
            Align T
          </button>
          <button disabled={selection.nodes.length < 2} onClick={() => alignSelected('vcenter')}>
            Align Y
          </button>
          <button disabled={selection.nodes.length < 2} onClick={() => alignSelected('bottom')}>
            Align B
          </button>
          <span className="divider" />
          <button disabled={selection.nodes.length < 3} onClick={() => distributeSelected('x')}>
            Distribute H
          </button>
          <button disabled={selection.nodes.length < 3} onClick={() => distributeSelected('y')}>
            Distribute V
          </button>
        </div>
        <div className="canvas-frame">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onSelectionChange={setSelection}
            fitView
            multiSelectionKeyCode={['Shift', 'Meta']}
            selectionOnDrag
            nodeOrigin={[0, 0]}
          >
            <Background color="#cbd5e1" gap={24} />
            <MiniMap pannable zoomable />
            <Controls />
          </ReactFlow>
        </div>
        <div className="operation-log">
          {operationLog.length === 0 ? (
            <code>No emitted operations yet.</code>
          ) : (
            operationLog
              .slice()
              .reverse()
              .map((operation) => (
                <code key={operation.opId}>
                  {operation.opId} {operation.op}
                </code>
              ))
          )}
        </div>
      </section>

      <aside className="inspector">
        <section>
          <h2>Inspector</h2>
          {selection.nodes[0] ? (
            <InspectorNode node={selection.nodes[0]} />
          ) : selection.edges[0] ? (
            <InspectorEdge edge={selection.edges[0]} />
          ) : (
            <p>Select an element or connector.</p>
          )}
        </section>
        <section>
          <h2>View Metadata</h2>
          <pre>{JSON.stringify(viewMetadata, null, 2)}</pre>
        </section>
        <section>
          <h2>Proposal Operations</h2>
          <div className="proposal-actions">
            <button disabled={operationLog.length === 0} onClick={saveProposalDraft}>
              Save draft
            </button>
            <button onClick={loadProposalDraft}>Load draft</button>
            <button disabled={operationLog.length === 0} onClick={() => setProposalState('accepted')}>
              Accept
            </button>
            <button disabled={operationLog.length === 0} onClick={downloadProposalDraft}>
              Download
            </button>
            <button disabled={operationLog.length === 0} onClick={clearProposalDraft}>
              Clear
            </button>
          </div>
          <p className="proposal-status">{proposalStatus}</p>
          <pre>{JSON.stringify(proposalDraft, null, 2)}</pre>
        </section>
      </aside>
    </main>
  );
}

function InspectorNode({ node }: { node: Node<RedshieldNodeData> }) {
  return (
    <dl className="inspector-list">
      <dt>ID</dt>
      <dd>{node.data.modelId}</dd>
      <dt>Kind</dt>
      <dd>{node.data.kind}</dd>
      <dt>Renderer</dt>
      <dd>{node.data.render.rendererId}</dd>
      <dt>Rule</dt>
      <dd>{node.data.matchedRuleId}</dd>
      <dt>Asset</dt>
      <dd>{node.data.asset ? `${node.data.asset.id} (${node.data.asset.status})` : 'none'}</dd>
      <dt>Label</dt>
      <dd>{node.data.label}</dd>
      <dt>Stereotypes</dt>
      <dd>{node.data.stereotypes.length > 0 ? node.data.stereotypes.join(', ') : 'none'}</dd>
      <dt>Layout</dt>
      <dd>{node.data.layoutState}</dd>
      <dt>Bounds</dt>
      <dd>
        {node.data.bounds.width} x {node.data.bounds.height}
      </dd>
      <dt>Position</dt>
      <dd>
        {Math.round(node.position.x)}, {Math.round(node.position.y)}
      </dd>
      <dt>Notes</dt>
      <dd>{node.data.description}</dd>
    </dl>
  );
}

function InspectorEdge({ edge }: { edge: Edge<RedshieldEdgeData> }) {
  return (
    <dl className="inspector-list">
      <dt>ID</dt>
      <dd>{edge.data?.relationshipId ?? edge.id}</dd>
      <dt>Kind</dt>
      <dd>{edge.data?.relationshipKind ?? 'association'}</dd>
      <dt>Route</dt>
      <dd>{edge.data?.routeHint ?? edge.type}</dd>
      <dt>Source</dt>
      <dd>{edge.source}</dd>
      <dt>Target</dt>
      <dd>{edge.target}</dd>
      <dt>Trace links</dt>
      <dd>{edge.data?.traceCount ?? 0}</dd>
    </dl>
  );
}
