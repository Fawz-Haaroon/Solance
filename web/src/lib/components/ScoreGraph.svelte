<script lang="ts">
    import type { MoveResponse } from '../types/analysis'

    const { moves, onMoveClick }: {
        moves:        MoveResponse[]
        onMoveClick?: (index: number) => void
    } = $props()

    const BAR_MAX = 500

    // Returns % from top (0 = top, 100 = bottom).
    // Positive score = bar extends upward from midpoint (50%).
    // Negative score = bar extends downward from midpoint.
    function whiteSegment(cp: number | null): { top: number; height: number } {
        const clamped = Math.min(Math.max(cp ?? 0, -BAR_MAX), BAR_MAX)
        if (clamped >= 0) {
            const h = (clamped / BAR_MAX) * 50
            return { top: 50 - h, height: h }
        } else {
            const h = (-clamped / BAR_MAX) * 50
            return { top: 50, height: h }
        }
    }

    function classColor(cls: string, cp: number | null): string {
        if (cp !== null && cp < 0) return '#334'
        switch (cls) {
            case 'best':       return '#5cb85c'
            case 'excellent':  return '#8bc34a'
            case 'good':       return '#aed581'
            case 'inaccuracy': return '#ffb74d'
            case 'mistake':    return '#ff7043'
            case 'blunder':    return '#e53935'
            default:           return '#5cb85c'
        }
    }
</script>

<div class="graph-wrap">
    <span class="axis-label top">+5</span>
    <span class="axis-label mid">0</span>
    <span class="axis-label bot">-5</span>

    <div class="bars">
        {#each moves as mv, i}
            {@const seg = whiteSegment(mv.score_cp)}
            <button
                class="bar-col"
                title="{mv.move_number}{mv.side === 'white' ? 'W' : 'B'} {mv.san} ({mv.loss_cp}cp loss)"
                onclick={() => onMoveClick?.(i)}
            >
                <div
                    class="bar-fill"
                    style="top: {seg.top}%; height: {Math.max(seg.height, 1)}%; background: {classColor(mv.class, mv.score_cp)};"
                ></div>
            </button>
        {/each}
    </div>

    <div class="midline"></div>
</div>

<style>
    .graph-wrap {
        position: relative;
        width: 100%;
        height: 140px;
        background: #0f0f1e;
        border-radius: 8px;
        overflow: hidden;
        border: 1px solid #1e1e36;
    }
    .bars {
        display: flex;
        align-items: stretch;
        height: 100%;
        gap: 1px;
        padding: 0 2px;
    }
    .bar-col {
        flex: 1;
        position: relative;
        background: none;
        border: none;
        cursor: pointer;
        padding: 0;
    }
    .bar-col:hover .bar-fill { filter: brightness(1.3); }
    .bar-fill {
        position: absolute;
        left: 1px;
        right: 1px;
        border-radius: 1px;
        transition: filter 0.1s;
    }
    .midline {
        position: absolute;
        top: 50%;
        left: 0;
        right: 0;
        height: 1px;
        background: rgba(255,255,255,0.12);
        pointer-events: none;
    }
    .axis-label {
        position: absolute;
        left: 5px;
        font-size: 9px;
        color: rgba(255,255,255,0.2);
        font-family: monospace;
        pointer-events: none;
        line-height: 1;
    }
    .axis-label.top { top:  4px; }
    .axis-label.mid { top: calc(50% - 5px); }
    .axis-label.bot { bottom: 4px; }
</style>
