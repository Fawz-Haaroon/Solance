<script lang="ts">
    import type { MoveResponse } from '../types/analysis'

    const { moves }: { moves: MoveResponse[] } = $props()

    const BAR_MAX = 500

    function barHeight(cp: number | null): number {
        if (cp === null) return 50
        return 50 + (Math.min(Math.max(cp, -BAR_MAX), BAR_MAX) / BAR_MAX) * 50
    }

    function classColor(cls: string): string {
        switch (cls) {
            case 'best':       return '#5cb85c'
            case 'excellent':  return '#8bc34a'
            case 'good':       return '#aed581'
            case 'inaccuracy': return '#ffb74d'
            case 'mistake':    return '#ff7043'
            case 'blunder':    return '#e53935'
            default:           return '#888'
        }
    }
</script>

<div class="graph-wrap">
    <div class="axis-label top">+5</div>
    <div class="bars">
        {#each moves as mv}
            <div
                class="bar-slot"
                title="{mv.move_number}{mv.side === 'white' ? 'W' : 'B'} {mv.san} ({mv.loss_cp}cp loss)"
            >
                <div
                    class="bar"
                    style="height: {barHeight(mv.score_cp)}%; background: {classColor(mv.class)};"
                ></div>
            </div>
        {/each}
    </div>
    <div class="axis-label bottom">-5</div>
    <div class="midline"></div>
</div>

<style>
    .graph-wrap {
        position: relative;
        width: 100%;
        height: 120px;
        background: #1a1a2e;
        border-radius: 6px;
        overflow: hidden;
        margin: 1.5rem 0;
    }
    .bars {
        display: flex;
        align-items: flex-end;
        height: 100%;
        gap: 1px;
        padding: 0 4px;
    }
    .bar-slot {
        flex: 1;
        height: 100%;
        display: flex;
        align-items: flex-end;
    }
    .bar {
        width: 100%;
        min-height: 2px;
        transition: height 0.2s ease;
        border-radius: 2px 2px 0 0;
    }
    .midline {
        position: absolute;
        top: 50%;
        left: 0;
        right: 0;
        height: 1px;
        background: rgba(255,255,255,0.15);
        pointer-events: none;
    }
    .axis-label {
        position: absolute;
        left: 4px;
        font-size: 10px;
        color: rgba(255,255,255,0.3);
        font-family: monospace;
    }
    .axis-label.top    { top: 2px; }
    .axis-label.bottom { bottom: 2px; }
</style>
