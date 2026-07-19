import { useCallback, useMemo, useState } from 'react';
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
import traceFile from '../../examples/minimal/redshield/trace/links.json';

type ElementRecord = (typeof elementsFile.elements)[number];
type RelationshipRecord = (typeof relationshipsFile.relationships)[number];
type DiagramLayout = NonNullable<(typeof diagramsFile.diagrams)[number]['layout']>;
type DiagramNodeLayout = DiagramLayout['nodes'][number];
type DiagramConnectorLayout = DiagramLayout['connectors'][number];
type RedshieldNodeData = {
  label: string;
  modelId: string;
  kind: string;
  description: string;
  tags: string[];
  layoutState: 'generated' | 'manual';
  bounds: { width: number; height: number };
  labelPosition?: { x: number; y: number };
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

const elk = new ELK();
const diagram = diagramsFile.diagrams[0];
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
  return {
    label: element.name,
    modelId: element.id,
    kind: element.kind,
    description: element.description,
    tags: element.tags,
    layoutState: toLayoutState(layout?.layoutState),
    bounds: {
      width: layout?.bounds.width ?? 210,
      height: layout?.bounds.height ?? 86,
    },
    labelPosition: layout?.labelPosition,
  };
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
  return (
    <div className={`diagram-node diagram-node--${data.kind} ${selected ? 'is-selected' : ''}`}>
      <Handle type="target" position={Position.Left} />
      <div className="diagram-node__kind">{data.kind.replace('_', ' ')}</div>
      <div className="diagram-node__label">{data.label}</div>
      <div className="diagram-node__id">{data.modelId}</div>
      <Handle type="source" position={Position.Right} />
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
  const [operationLog, setOperationLog] = useState<string[]>([
    'Loaded semantic model package from examples/minimal/redshield.',
  ]);

  const selectedNodeIds = useMemo(
    () => new Set(selection.nodes.map((node) => node.id)),
    [selection.nodes],
  );

  const onNodesChange = useCallback((changes: NodeChange<Node<RedshieldNodeData>>[]) => {
    setNodes((currentNodes) => {
      const changedIds = new Set(
        changes
          .filter((change) => change.type === 'position' && change.dragging === false)
          .map((change) => ('id' in change ? change.id : '')),
      );
      return applyNodeChanges(changes, currentNodes).map((node) =>
        changedIds.has(node.id)
          ? { ...node, data: { ...node.data, layoutState: 'manual' } }
          : node,
      );
    });
  }, []);

  const onEdgesChange = useCallback((changes: EdgeChange<Edge<RedshieldEdgeData>>[]) => {
    setEdges((currentEdges) => applyEdgeChanges(changes, currentEdges));
  }, []);

  const onConnect = useCallback((connection: Connection) => {
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
    pushOperation(`create_relationship draft ${connection.source} -> ${connection.target}`);
  }, []);

  const pushOperation = useCallback((entry: string) => {
    setOperationLog((current) => [entry, ...current].slice(0, 8));
  }, []);

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
    pushOperation('apply_auto_layout elk.layered');
  }, [edges, nodes, pushOperation]);

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
      pushOperation(`align_diagram_elements ${mode} ${Array.from(selectedNodeIds).join(', ')}`);
    },
    [nodes, pushOperation, selectedNodeIds],
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
      pushOperation(`distribute_diagram_elements ${axis} ${Array.from(selectedNodeIds).join(', ')}`);
    },
    [nodes, pushOperation, selectedNodeIds],
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
          {operationLog.map((entry) => (
            <code key={entry}>{entry}</code>
          ))}
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
      <dt>Label</dt>
      <dd>{node.data.label}</dd>
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
