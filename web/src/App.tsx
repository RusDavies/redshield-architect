import { useCallback, useEffect, useMemo, useRef, useState, type CSSProperties } from 'react';
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
type ExternalReference = {
  id: string;
  label: string;
  uri: string;
  kind?: string;
};
type ArchitectureOwner = {
  ref: string;
  role?: string;
  name?: string;
};
type ArchitectureLifecycle = {
  state?: string;
  phase?: string;
  milestoneRefs?: string[];
  targetDate?: string;
  notes?: string;
};
type TechnologyMapping = {
  ref: string;
  role?: string;
  version?: string;
  standardState?: string;
};
type RiskMapping = {
  ref: string;
  severity?: string;
  status?: string;
  notes?: string;
};
type CapabilityMapping = {
  ref: string;
  fit?: string;
  maturity?: string;
};
type ServiceMapping = {
  ref: string;
  relationship?: string;
  interfaceRef?: string;
};
type ArchitectureDetails = {
  owners?: ArchitectureOwner[];
  lifecycle?: ArchitectureLifecycle;
  criticality?: string;
  technologies?: TechnologyMapping[];
  risks?: RiskMapping[];
  capabilities?: CapabilityMapping[];
  services?: ServiceMapping[];
};
type Multiplicity = {
  lower?: number;
  upper?: number | string;
  isOrdered?: boolean;
  isUnique?: boolean;
};
type ClassifierAttribute = {
  name: string;
  visibility?: string;
  typeRef?: string;
  multiplicity?: Multiplicity;
  defaultValue?: string;
  isStatic?: boolean;
  isReadOnly?: boolean;
  documentation?: string;
};
type OperationParameter = {
  name: string;
  typeRef?: string;
  direction?: string;
  multiplicity?: Multiplicity;
  defaultValue?: string;
};
type ClassifierOperation = {
  name: string;
  visibility?: string;
  returnTypeRef?: string;
  parameters?: OperationParameter[];
  isAbstract?: boolean;
  isStatic?: boolean;
  documentation?: string;
};
type ClassifierDetails = {
  isAbstract?: boolean;
  isStatic?: boolean;
  attributes?: ClassifierAttribute[];
  operations?: ClassifierOperation[];
};
type ActorDetails = {
  actorType?: string;
  responsibilities?: string[];
  goals?: string[];
  constraints?: string[];
};
type UseCaseStep = {
  step: number;
  actorRef?: string;
  action: string;
};
type UseCaseAlternateFlow = {
  name: string;
  trigger?: string;
  steps?: UseCaseStep[];
};
type UseCaseDetails = {
  primaryActorRef?: string;
  supportingActorRefs?: string[];
  preconditions?: string[];
  postconditions?: string[];
  mainFlow?: UseCaseStep[];
  alternateFlows?: UseCaseAlternateFlow[];
  extensionPoints?: string[];
};
type ActivityParameter = {
  name: string;
  typeRef?: string;
  direction?: string;
};
type ActivityNode = {
  id: string;
  name: string;
  kind: string;
  description?: string;
};
type ActivityFlow = {
  id: string;
  sourceNodeId: string;
  targetNodeId: string;
  guard?: string;
};
type ActivityDetails = {
  parameters?: ActivityParameter[];
  nodes?: ActivityNode[];
  flows?: ActivityFlow[];
};
type SequenceParticipantDetails = {
  participantKind?: string;
  representsRef?: string;
  lifelineName?: string;
  isExternal?: boolean;
};
type RedshieldNodeData = {
  label: string;
  modelId: string;
  kind: string;
  aliases: string[];
  description: string;
  documentation: string;
  status: string;
  stereotypes: string[];
  tags: string[];
  externalReferences: ExternalReference[];
  architecture?: ArchitectureDetails;
  classifier?: ClassifierDetails;
  actorDetails?: ActorDetails;
  useCaseDetails?: UseCaseDetails;
  activityDetails?: ActivityDetails;
  sequenceParticipantDetails?: SequenceParticipantDetails;
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
const defaultRenderProfile = renderProfileFile.profiles[0] as RenderProfile;
const proposalStorageKey = `redshield.workbench.${diagram.id}.proposalDraft`;
const elementById = new Map(elementsFile.elements.map((element) => [element.id, element]));
const nodeLayoutByRef = new Map(
  (diagram.layout?.nodes ?? []).map((nodeLayout) => [nodeLayout.modelRef, nodeLayout]),
);
const connectorLayoutByRef = new Map(
  (diagram.layout?.connectors ?? []).map((connectorLayout) => [
    connectorLayout.relationshipRef,
    connectorLayout,
  ]),
);

function initialNodes(profile: RenderProfile): Node<RedshieldNodeData>[] {
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
        data: toNodeData(element, profile, persisted),
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

function toNodeData(
  element: ElementRecord,
  profile: RenderProfile,
  layout?: DiagramNodeLayout,
): RedshieldNodeData {
  const resolution = resolveRenderTarget(element, profile);
  const renderAssetById = new Map((profile.assets ?? []).map((asset) => [asset.id, asset]));
  return {
    label: element.name,
    modelId: element.id,
    kind: element.kind,
    aliases: element.aliases ?? [],
    description: element.description,
    documentation: element.documentation ?? '',
    status: element.status ?? 'accepted',
    stereotypes: element.stereotypes ?? [],
    tags: element.tags,
    externalReferences: element.externalReferences ?? [],
    architecture: element.architecture,
    classifier: element.classifier,
    actorDetails: element.actorDetails,
    useCaseDetails: element.useCaseDetails,
    activityDetails: element.activityDetails,
    sequenceParticipantDetails: element.sequenceParticipantDetails,
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

function resolveRenderTarget(
  element: ElementRecord,
  profile: RenderProfile,
): { render: RenderTarget; ruleId: string } {
  const rule = profile.rules
    .filter((candidate) => candidate.enabled !== false && matchesSelector(element, candidate))
    .sort((left, right) => right.precedence - left.precedence)[0];

  return rule
    ? { render: rule.renderAs, ruleId: rule.id }
    : { render: profile.fallback, ruleId: 'fallback' };
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
type SelectorMode = keyof RenderSelector;

const selectorModes: { value: SelectorMode; label: string }[] = [
  { value: 'elementId', label: 'ID' },
  { value: 'elementKind', label: 'Kind' },
  { value: 'stereotype', label: 'Stereotype' },
  { value: 'tag', label: 'Tag' },
];

const rendererOptions: RendererId[] = [
  'uml.actor',
  'uml.use_case',
  'uml.class',
  'uml.component',
  'uml.activity',
  'uml.sequence_participant',
  'image.element',
];

const colorSwatches = ['#ffffff', '#ccfbf1', '#e0f2fe', '#fef3c7', '#fee2e2', '#ede9fe'];

function selectorValueOptions(mode: SelectorMode) {
  if (mode === 'elementId') {
    return elementsFile.elements.map((element) => ({
      value: element.id,
      label: element.name,
    }));
  }
  if (mode === 'elementKind') {
    return Array.from(new Set(elementsFile.elements.map((element) => element.kind))).map((kind) => ({
      value: kind,
      label: kind.replace('_', ' '),
    }));
  }
  if (mode === 'stereotype') {
    return Array.from(new Set(elementsFile.elements.flatMap((element) => element.stereotypes ?? []))).map(
      (stereotype) => ({ value: stereotype, label: stereotype }),
    );
  }
  return Array.from(new Set(elementsFile.elements.flatMap((element) => element.tags))).map((tag) => ({
    value: tag,
    label: tag,
  }));
}

function sanitizeRuleIdPart(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function defaultPortsForRenderer(rendererId: RendererId): RenderPort[] {
  if (rendererId === 'uml.actor') return [{ id: 'out', side: 'right', offset: 0.5 }];
  return [
    { id: 'in', side: 'left', offset: 0.5 },
    { id: 'out', side: 'right', offset: 0.5 },
  ];
}

export default function App() {
  const [renderProfile, setRenderProfile] = useState<RenderProfile>(() => defaultRenderProfile);
  const [nodes, setNodes] = useState<Node<RedshieldNodeData>[]>(() =>
    initialNodes(defaultRenderProfile),
  );
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
  const [renderProfileStatus, setRenderProfileStatus] = useState('Default render profile loaded.');

  const selectedNodeIds = useMemo(
    () => new Set(selection.nodes.map((node) => node.id)),
    [selection.nodes],
  );
  const selectedNode = selection.nodes[0];
  const renderProfilePreview = useMemo(
    () => ({
      schemaVersion: renderProfileFile.schemaVersion,
      profiles: [renderProfile],
    }),
    [renderProfile],
  );

  useEffect(() => {
    setNodes((currentNodes) =>
      currentNodes.map((node) => {
        const element = elementById.get(node.id);
        if (!element) return node;
        return {
          ...node,
          data: toNodeData(element, renderProfile, {
            modelRef: node.id,
            bounds: {
              x: node.position.x,
              y: node.position.y,
              width: node.data.bounds.width,
              height: node.data.bounds.height,
            },
            layoutState: node.data.layoutState,
            labelPosition: node.data.labelPosition,
          }),
        };
      }),
    );
  }, [renderProfile]);

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
      intent: 'Apply direct manipulation and render profile changes from the workbench.',
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
  const downloadRenderProfile = useCallback(() => {
    const blob = new Blob([`${JSON.stringify(renderProfilePreview, null, 2)}\n`], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'render-profile.workbench-draft.json';
    anchor.click();
    URL.revokeObjectURL(url);
    setRenderProfileStatus('Downloaded draft render profile JSON.');
  }, [renderProfilePreview]);
  const applyRenderRule = useCallback(
    (rule: RenderRule) => {
      setRenderProfile((current) => {
        const nextRules = current.rules.filter((currentRule) => currentRule.id !== rule.id);
        return { ...current, rules: [...nextRules, rule] };
      });
      recordOperation({
        op: 'upsert_render_rule',
        args: {
          profileId: renderProfile.id,
          rule,
        },
        rationale: 'Workbench render-rule editor changed renderer/profile metadata.',
        sourceRefs: ['workbench.render-rules'],
      });
      setRenderProfileStatus(`Applied ${rule.id} locally.`);
    },
    [recordOperation, renderProfile.id],
  );
  const toggleRenderRule = useCallback(
    (ruleId: string) => {
      const rule = renderProfile.rules.find((currentRule) => currentRule.id === ruleId);
      if (!rule) return;
      const nextRule = { ...rule, enabled: rule.enabled === false };
      setRenderProfile((current) => ({
        ...current,
        rules: current.rules.map((currentRule) =>
          currentRule.id === ruleId ? nextRule : currentRule,
        ),
      }));
      recordOperation({
        op: 'upsert_render_rule',
        args: {
          profileId: renderProfile.id,
          rule: nextRule,
        },
        rationale: 'Workbench render-rule editor toggled a renderer rule.',
        sourceRefs: ['workbench.render-rules'],
      });
      setRenderProfileStatus(`Toggled ${ruleId}.`);
    },
    [recordOperation, renderProfile],
  );

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
        <RenderRuleEditor
          profile={renderProfile}
          selectedNode={selectedNode}
          status={renderProfileStatus}
          onApply={applyRenderRule}
          onToggle={toggleRenderRule}
          onReset={() => {
            setRenderProfile(defaultRenderProfile);
            setRenderProfileStatus('Reset to the packaged default render profile.');
          }}
          onDownload={downloadRenderProfile}
        />
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
        <section>
          <h2>Render Profile</h2>
          <pre>{JSON.stringify(renderProfilePreview, null, 2)}</pre>
        </section>
      </aside>
    </main>
  );
}

function RenderRuleEditor({
  profile,
  selectedNode,
  status,
  onApply,
  onToggle,
  onReset,
  onDownload,
}: {
  profile: RenderProfile;
  selectedNode?: Node<RedshieldNodeData>;
  status: string;
  onApply: (rule: RenderRule) => void;
  onToggle: (ruleId: string) => void;
  onReset: () => void;
  onDownload: () => void;
}) {
  const [selectorMode, setSelectorMode] = useState<SelectorMode>('elementKind');
  const [selectorValue, setSelectorValue] = useState('class');
  const [rendererId, setRendererId] = useState<RendererId>('uml.class');
  const [assetRef, setAssetRef] = useState(profile.assets?.[0]?.id ?? '');
  const [fillColor, setFillColor] = useState('#ffffff');
  const [strokeColor, setStrokeColor] = useState('#334155');
  const [textColor, setTextColor] = useState('#0f172a');
  const [precedence, setPrecedence] = useState(150);

  const options = selectorValueOptions(selectorMode);
  const canApply = selectorValue.trim().length > 0 && rendererId !== 'html.custom';

  const applySelectedNode = () => {
    if (!selectedNode) return;
    setSelectorMode('elementId');
    setSelectorValue(selectedNode.id);
    setRendererId(selectedNode.data.render.rendererId);
    setAssetRef(selectedNode.data.render.assetRef ?? profile.assets?.[0]?.id ?? '');
    setFillColor(selectedNode.data.render.style?.fillColor ?? fillColor);
    setStrokeColor(selectedNode.data.render.style?.strokeColor ?? strokeColor);
    setTextColor(selectedNode.data.render.style?.textColor ?? textColor);
  };

  const applyRule = () => {
    if (!canApply) return;
    const selector = { [selectorMode]: selectorValue.trim() } as RenderSelector;
    const style =
      rendererId === 'image.element'
        ? undefined
        : {
            fillColor,
            strokeColor,
            textColor,
          };
    onApply({
      id: `render.ui.${selectorMode}.${sanitizeRuleIdPart(selectorValue)}`,
      description: `Workbench-authored rule for ${selectorMode} ${selectorValue}.`,
      selector,
      renderAs: {
        rendererId,
        assetRef: rendererId === 'image.element' ? assetRef : undefined,
        style,
        ports: defaultPortsForRenderer(rendererId),
        label: {
          visible: true,
          position: rendererId === 'uml.actor' || rendererId === 'image.element' ? 'bottom' : 'inside',
        },
      },
      precedence,
      enabled: true,
    });
  };

  return (
    <section>
      <h2>Render Rules</h2>
      <div className="rule-editor">
        <div className="segmented-control" aria-label="Selector type">
          {selectorModes.map((mode) => (
            <button
              key={mode.value}
              className={selectorMode === mode.value ? 'is-active' : ''}
              onClick={() => {
                setSelectorMode(mode.value);
                setSelectorValue(selectorValueOptions(mode.value)[0]?.value ?? '');
              }}
              type="button"
            >
              {mode.label}
            </button>
          ))}
        </div>
        <label>
          Match
          <select
            onChange={(event) => setSelectorValue(event.target.value)}
            value={selectorValue}
          >
            {options.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          Renderer
          <select
            onChange={(event) => setRendererId(event.target.value as RendererId)}
            value={rendererId}
          >
            {rendererOptions.map((renderer) => (
              <option key={renderer} value={renderer}>
                {renderer}
              </option>
            ))}
          </select>
        </label>
        {rendererId === 'image.element' ? (
          <label>
            Asset
            <select onChange={(event) => setAssetRef(event.target.value)} value={assetRef}>
              {(profile.assets ?? []).map((asset) => (
                <option key={asset.id} value={asset.id}>
                  {asset.id} ({asset.status})
                </option>
              ))}
            </select>
          </label>
        ) : (
          <div className="swatch-grid" aria-label="Fill color">
            {colorSwatches.map((color) => (
              <button
                key={color}
                aria-label={`Fill ${color}`}
                className={fillColor === color ? 'is-active' : ''}
                onClick={() => setFillColor(color)}
                style={{ background: color }}
                type="button"
              />
            ))}
          </div>
        )}
        <div className="color-inputs">
          <label>
            Fill
            <input onChange={(event) => setFillColor(event.target.value)} type="color" value={fillColor} />
          </label>
          <label>
            Stroke
            <input
              onChange={(event) => setStrokeColor(event.target.value)}
              type="color"
              value={strokeColor}
            />
          </label>
          <label>
            Text
            <input onChange={(event) => setTextColor(event.target.value)} type="color" value={textColor} />
          </label>
        </div>
        <label>
          Precedence
          <input
            max="500"
            min="0"
            onChange={(event) => setPrecedence(Number(event.target.value))}
            type="range"
            value={precedence}
          />
          <span>{precedence}</span>
        </label>
        <div className="rule-actions">
          <button disabled={!selectedNode} onClick={applySelectedNode} type="button">
            Use selection
          </button>
          <button disabled={!canApply} onClick={applyRule} type="button">
            Apply rule
          </button>
          <button onClick={onDownload} type="button">
            Download
          </button>
          <button onClick={onReset} type="button">
            Reset
          </button>
        </div>
        <p className="proposal-status">{status}</p>
      </div>
      <div className="rule-list">
        {profile.rules
          .slice()
          .sort((left, right) => right.precedence - left.precedence)
          .map((rule) => (
            <button
              key={rule.id}
              className={rule.enabled === false ? 'is-disabled' : ''}
              onClick={() => onToggle(rule.id)}
              type="button"
            >
              <span>{rule.id}</span>
              <small>
                {Object.entries(rule.selector)
                  .map(([key, value]) => `${key}:${value}`)
                  .join(' ')}{' '}
                {' -> '}
                {rule.renderAs.rendererId}
              </small>
            </button>
          ))}
      </div>
    </section>
  );
}

function InspectorNode({ node }: { node: Node<RedshieldNodeData> }) {
  const attributes = node.data.classifier?.attributes ?? [];
  const operations = node.data.classifier?.operations ?? [];

  return (
    <dl className="inspector-list">
      <dt>ID</dt>
      <dd>{node.data.modelId}</dd>
      <dt>Kind</dt>
      <dd>{node.data.kind}</dd>
      <dt>Status</dt>
      <dd>{node.data.status}</dd>
      <dt>Renderer</dt>
      <dd>{node.data.render.rendererId}</dd>
      <dt>Rule</dt>
      <dd>{node.data.matchedRuleId}</dd>
      <dt>Asset</dt>
      <dd>{node.data.asset ? `${node.data.asset.id} (${node.data.asset.status})` : 'none'}</dd>
      <dt>Label</dt>
      <dd>{node.data.label}</dd>
      <dt>Aliases</dt>
      <dd>{node.data.aliases.length > 0 ? node.data.aliases.join(', ') : 'none'}</dd>
      <dt>Stereotypes</dt>
      <dd>{node.data.stereotypes.length > 0 ? node.data.stereotypes.join(', ') : 'none'}</dd>
      <dt>External</dt>
      <dd>
        {node.data.externalReferences.length > 0
          ? node.data.externalReferences.map((reference) => reference.label).join(', ')
          : 'none'}
      </dd>
      <dt>Classifier</dt>
      <dd>
        {node.data.classifier
          ? [
              node.data.classifier.isAbstract ? 'abstract' : undefined,
              node.data.classifier.isStatic ? 'static' : undefined,
              `${attributes.length} attr`,
              `${operations.length} ops`,
            ]
              .filter(Boolean)
              .join(', ')
          : 'none'}
      </dd>
      {attributes.length > 0 ? (
        <>
          <dt>Attributes</dt>
          <dd>{attributes.map(formatAttribute).join('; ')}</dd>
        </>
      ) : null}
      {operations.length > 0 ? (
        <>
          <dt>Operations</dt>
          <dd>{operations.map(formatOperation).join('; ')}</dd>
        </>
      ) : null}
      <SemanticDetails node={node.data} />
      <ArchitectureDetailsView architecture={node.data.architecture} />
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
      <dt>Docs</dt>
      <dd>{node.data.documentation || 'none'}</dd>
    </dl>
  );
}

function SemanticDetails({ node }: { node: RedshieldNodeData }) {
  if (node.actorDetails) {
    return (
      <>
        <dt>Actor type</dt>
        <dd>{node.actorDetails.actorType ?? 'unspecified'}</dd>
        <dt>Responsibilities</dt>
        <dd>{formatList(node.actorDetails.responsibilities)}</dd>
        <dt>Goals</dt>
        <dd>{formatList(node.actorDetails.goals)}</dd>
        <dt>Constraints</dt>
        <dd>{formatList(node.actorDetails.constraints)}</dd>
      </>
    );
  }

  if (node.useCaseDetails) {
    return (
      <>
        <dt>Primary actor</dt>
        <dd>{node.useCaseDetails.primaryActorRef || 'none'}</dd>
        <dt>Preconditions</dt>
        <dd>{formatList(node.useCaseDetails.preconditions)}</dd>
        <dt>Main flow</dt>
        <dd>{formatUseCaseSteps(node.useCaseDetails.mainFlow)}</dd>
        <dt>Alternate flows</dt>
        <dd>{formatAlternateFlows(node.useCaseDetails.alternateFlows)}</dd>
        <dt>Postconditions</dt>
        <dd>{formatList(node.useCaseDetails.postconditions)}</dd>
        <dt>Extension points</dt>
        <dd>{formatList(node.useCaseDetails.extensionPoints)}</dd>
      </>
    );
  }

  if (node.activityDetails) {
    return (
      <>
        <dt>Parameters</dt>
        <dd>{formatActivityParameters(node.activityDetails.parameters)}</dd>
        <dt>Activity nodes</dt>
        <dd>{formatActivityNodes(node.activityDetails.nodes)}</dd>
        <dt>Activity flows</dt>
        <dd>{formatActivityFlows(node.activityDetails.flows)}</dd>
      </>
    );
  }

  if (node.sequenceParticipantDetails) {
    return (
      <>
        <dt>Participant</dt>
        <dd>
          {[
            node.sequenceParticipantDetails.participantKind,
            node.sequenceParticipantDetails.lifelineName,
            node.sequenceParticipantDetails.representsRef,
            node.sequenceParticipantDetails.isExternal ? 'external' : undefined,
          ]
            .filter(Boolean)
            .join(' / ') || 'unspecified'}
        </dd>
      </>
    );
  }

  return null;
}

function ArchitectureDetailsView({ architecture }: { architecture?: ArchitectureDetails }) {
  if (!architecture || !hasArchitectureDetails(architecture)) return null;

  return (
    <>
      <dt>Criticality</dt>
      <dd>{architecture.criticality || 'unspecified'}</dd>
      <dt>Lifecycle</dt>
      <dd>{formatLifecycle(architecture.lifecycle)}</dd>
      <dt>Owners</dt>
      <dd>{formatOwners(architecture.owners)}</dd>
      <dt>Technologies</dt>
      <dd>{formatTechnologies(architecture.technologies)}</dd>
      <dt>Risks</dt>
      <dd>{formatRisks(architecture.risks)}</dd>
      <dt>Capabilities</dt>
      <dd>{formatCapabilities(architecture.capabilities)}</dd>
      <dt>Services</dt>
      <dd>{formatServices(architecture.services)}</dd>
    </>
  );
}

function hasArchitectureDetails(architecture: ArchitectureDetails): boolean {
  return Boolean(
    architecture.criticality ||
      architecture.lifecycle ||
      architecture.owners?.length ||
      architecture.technologies?.length ||
      architecture.risks?.length ||
      architecture.capabilities?.length ||
      architecture.services?.length,
  );
}

function formatAttribute(attribute: ClassifierAttribute): string {
  const flags = [attribute.isStatic ? 'static' : undefined, attribute.isReadOnly ? 'read-only' : undefined]
    .filter(Boolean)
    .join(' ');
  const suffix = [
    attribute.typeRef ? `: ${attribute.typeRef}` : '',
    formatMultiplicity(attribute.multiplicity),
    attribute.defaultValue ? ` = ${attribute.defaultValue}` : '',
    flags ? ` (${flags})` : '',
  ].join('');
  return `${visibilitySymbol(attribute.visibility)}${attribute.name}${suffix}`;
}

function formatOperation(operation: ClassifierOperation): string {
  const parameters = (operation.parameters ?? [])
    .map((parameter) => {
      const direction = parameter.direction && parameter.direction !== 'in' ? `${parameter.direction} ` : '';
      const typeRef = parameter.typeRef ? `: ${parameter.typeRef}` : '';
      const defaultValue = parameter.defaultValue ? ` = ${parameter.defaultValue}` : '';
      return `${direction}${parameter.name}${typeRef}${formatMultiplicity(parameter.multiplicity)}${defaultValue}`;
    })
    .join(', ');
  const flags = [operation.isAbstract ? 'abstract' : undefined, operation.isStatic ? 'static' : undefined]
    .filter(Boolean)
    .join(' ');
  const returnType = operation.returnTypeRef ? `: ${operation.returnTypeRef}` : '';
  return `${visibilitySymbol(operation.visibility)}${operation.name}(${parameters})${returnType}${
    flags ? ` (${flags})` : ''
  }`;
}

function formatMultiplicity(multiplicity?: Multiplicity): string {
  if (!multiplicity) return '';
  const lower = multiplicity.lower ?? '';
  const upper = multiplicity.upper ?? '';
  const bounds = lower === '' && upper === '' ? '' : `[${lower}..${upper}]`;
  const qualifiers = [
    multiplicity.isOrdered ? 'ordered' : undefined,
    multiplicity.isUnique ? 'unique' : undefined,
  ]
    .filter(Boolean)
    .join(',');
  return `${bounds}${qualifiers ? `{${qualifiers}}` : ''}`;
}

function visibilitySymbol(visibility?: string): string {
  if (visibility === 'private') return '-';
  if (visibility === 'protected') return '#';
  if (visibility === 'package') return '~';
  return '+';
}

function formatList(values?: string[]): string {
  return values && values.length > 0 ? values.join('; ') : 'none';
}

function formatLifecycle(lifecycle?: ArchitectureLifecycle): string {
  if (!lifecycle) return 'none';
  return [
    lifecycle.state,
    lifecycle.phase,
    lifecycle.targetDate ? `target ${lifecycle.targetDate}` : undefined,
    formatList(lifecycle.milestoneRefs) !== 'none' ? `milestones ${formatList(lifecycle.milestoneRefs)}` : undefined,
    lifecycle.notes,
  ]
    .filter(Boolean)
    .join('; ');
}

function formatOwners(owners?: ArchitectureOwner[]): string {
  return owners && owners.length > 0
    ? owners.map((owner) => [owner.role, owner.name, owner.ref].filter(Boolean).join(' ')).join('; ')
    : 'none';
}

function formatTechnologies(technologies?: TechnologyMapping[]): string {
  return technologies && technologies.length > 0
    ? technologies
        .map((technology) =>
          [
            technology.ref,
            technology.role ? `as ${technology.role}` : undefined,
            technology.version ? `v${technology.version}` : undefined,
            technology.standardState,
          ]
            .filter(Boolean)
            .join(' '),
        )
        .join('; ')
    : 'none';
}

function formatRisks(risks?: RiskMapping[]): string {
  return risks && risks.length > 0
    ? risks
        .map((risk) => [risk.ref, risk.severity, risk.status, risk.notes].filter(Boolean).join(' / '))
        .join('; ')
    : 'none';
}

function formatCapabilities(capabilities?: CapabilityMapping[]): string {
  return capabilities && capabilities.length > 0
    ? capabilities
        .map((capability) => [capability.ref, capability.fit, capability.maturity].filter(Boolean).join(' / '))
        .join('; ')
    : 'none';
}

function formatServices(services?: ServiceMapping[]): string {
  return services && services.length > 0
    ? services
        .map((service) => [service.ref, service.relationship, service.interfaceRef].filter(Boolean).join(' / '))
        .join('; ')
    : 'none';
}

function formatUseCaseSteps(steps?: UseCaseStep[]): string {
  return steps && steps.length > 0
    ? steps.map((step) => `${step.step}. ${step.actorRef ? `${step.actorRef}: ` : ''}${step.action}`).join('; ')
    : 'none';
}

function formatAlternateFlows(flows?: UseCaseAlternateFlow[]): string {
  return flows && flows.length > 0
    ? flows
        .map((flow) => `${flow.name}${flow.trigger ? ` (${flow.trigger})` : ''}: ${formatUseCaseSteps(flow.steps)}`)
        .join('; ')
    : 'none';
}

function formatActivityParameters(parameters?: ActivityParameter[]): string {
  return parameters && parameters.length > 0
    ? parameters
        .map((parameter) =>
          [parameter.direction, parameter.name, parameter.typeRef ? `: ${parameter.typeRef}` : undefined]
            .filter(Boolean)
            .join(' '),
        )
        .join('; ')
    : 'none';
}

function formatActivityNodes(nodes?: ActivityNode[]): string {
  return nodes && nodes.length > 0
    ? nodes.map((node) => `${node.name} (${node.kind})`).join('; ')
    : 'none';
}

function formatActivityFlows(flows?: ActivityFlow[]): string {
  return flows && flows.length > 0
    ? flows
        .map((flow) => `${flow.sourceNodeId} -> ${flow.targetNodeId}${flow.guard ? ` [${flow.guard}]` : ''}`)
        .join('; ')
    : 'none';
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
