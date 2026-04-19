/**
 * Structure pane: renders projected nodes and hotspots.
 */
import { projection } from './editor-state';
import type { Hotspot } from './editor-state';
import { openPopup, popupState } from './popup-state';
import { NodePopup } from './NodePopup';

export function StructurePane() {
  const proj = projection.value;

  const hotspotsByParent = new Map<string, Hotspot[]>();
  for (const h of proj.hotspots) {
    const list = hotspotsByParent.get(h.parent_id) ?? [];
    list.push(h);
    hotspotsByParent.set(h.parent_id, list);
  }

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
        {proj.nodes.map(node => {
          const nodeHotspots = hotspotsByParent.get(node.id) ?? [];
          return (
            <div key={node.id} class="structure-node" style={{ paddingLeft: `${node.depth * 1.2}rem` }}>
              <span class={`node-label ${node.is_hole ? 'node-hole' : ''}`}>
                {node.label}
              </span>
              {nodeHotspots.map(h => (
                <HotspotButton key={`${h.direction}-${h.parent_id}`} hotspot={h} />
              ))}
            </div>
          );
        })}
        {/* Hotspots not tied to displayed nodes (e.g., Sequence below) */}
        {proj.nodes.length > 0 && proj.hotspots
          .filter(h => h.direction === 'below' && !proj.nodes.some(n => n.id === h.parent_id))
          .map(h => (
            <div key={`orphan-below-${h.parent_id}`} class="structure-node">
              <HotspotButton hotspot={h} />
            </div>
          ))
        }

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
