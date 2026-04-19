/**
 * Structure pane: renders projected nodes and hotspots.
 */
import { projection } from './editor-state';
import type { Hotspot } from './editor-state';
import { openPopup, popupState } from './popup-state';
import { NodePopup } from './NodePopup';

type RenderItem =
  | { type: 'node'; node: { id: string; label: string; depth: number; is_hole: boolean }; hotspots: Hotspot[] }
  | { type: 'below'; hotspot: Hotspot; depth: number };

/**
 * Build an interleaved list of nodes and deferred "below" hotspots.
 * "below" hotspots are placed after their parent's subtree ends (not inline
 * with the parent node), so that DOM order matches visual nesting order.
 */
function buildRenderItems(
  nodes: { id: string; label: string; depth: number; is_hole: boolean }[],
  hotspotsByParent: Map<string, Hotspot[]>,
): RenderItem[] {
  const items: RenderItem[] = [];
  const pendingBelow: { depth: number; hotspot: Hotspot }[] = [];

  for (const node of nodes) {
    // Flush pending "below" hotspots whose subtree just ended
    while (pendingBelow.length > 0) {
      const top = pendingBelow[pendingBelow.length - 1];
      if (top.depth >= node.depth) {
        pendingBelow.pop();
        items.push({ type: 'below', hotspot: top.hotspot, depth: top.depth + 1 });
      } else {
        break;
      }
    }

    const nodeHotspots = hotspotsByParent.get(node.id) ?? [];
    const belowHotspot = nodeHotspots.find(h => h.direction === 'below');
    const otherHotspots = nodeHotspots.filter(h => h.direction !== 'below');

    items.push({ type: 'node', node, hotspots: otherHotspots });

    if (belowHotspot) {
      pendingBelow.push({ depth: node.depth, hotspot: belowHotspot });
    }
  }

  // Flush remaining (deepest first)
  while (pendingBelow.length > 0) {
    const top = pendingBelow.pop()!;
    items.push({ type: 'below', hotspot: top.hotspot, depth: top.depth + 1 });
  }

  return items;
}

export function StructurePane() {
  const proj = projection.value;

  const hotspotsByParent = new Map<string, Hotspot[]>();
  for (const h of proj.hotspots) {
    const list = hotspotsByParent.get(h.parent_id) ?? [];
    list.push(h);
    hotspotsByParent.set(h.parent_id, list);
  }

  const items = proj.nodes.length > 0
    ? buildRenderItems(proj.nodes, hotspotsByParent)
    : [];

  return (
    <div class="pane" data-testid="structure-pane">
      <div class="pane-header">
        <span class="pane-title">Structure</span>
      </div>
      <div class="pane-content-scroll">
        {proj.nodes.length === 0 && (
          <div class="structure-empty">
            {proj.hotspots.filter(h => h.direction === 'below').map(h => (
              <HotspotButton key={`below-${h.parent_id}`} hotspot={h} />
            ))}
          </div>
        )}
        {items.map(item => {
          if (item.type === 'node') {
            return (
              <div key={item.node.id} class="structure-node" style={{ paddingLeft: `${item.node.depth * 1.2}rem` }}>
                <span class={`node-label ${item.node.is_hole ? 'node-hole' : ''}`}>
                  {item.node.label}
                </span>
                {item.hotspots.map(h => (
                  <HotspotButton key={`${h.direction}-${h.parent_id}`} hotspot={h} />
                ))}
              </div>
            );
          }
          return (
            <div key={`below-${item.hotspot.parent_id}`} class="structure-node" style={{ paddingLeft: `${item.depth * 1.2}rem` }}>
              <HotspotButton hotspot={item.hotspot} />
            </div>
          );
        })}

        {popupState.value.step !== 'closed' && <NodePopup />}
      </div>
    </div>
  );
}

function HotspotButton({ hotspot }: { hotspot: Hotspot }) {
  return (
    <button
      class={`hotspot-btn hotspot-${hotspot.direction}`}
      data-testid={`insertion-hotspot-${hotspot.direction}`}
      onClick={() => openPopup(hotspot)}
    >
      {hotspot.direction === 'below' && '＋↓'}
      {hotspot.direction === 'right' && '＋→'}
      {hotspot.direction === 'inside' && '＋◇'}
      {hotspot.direction === 'variant' && '＋⑅'}
    </button>
  );
}
