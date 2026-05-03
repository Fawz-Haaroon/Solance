<script lang="ts">
    import type { MoveResponse } from '../types/analysis'

    const { moves, onMoveClick }: {
        moves:        MoveResponse[]
        onMoveClick?: (index: number) => void
    } = $props()

    const BAR_MAX = 600

    function whiteSegment(cp: number | null): { top: number; height: number } {
        const raw     = cp ?? 0
        const clamped = Math.min(Math.max(raw, -BAR_MAX), BAR_MAX)
        if (clamped >= 0) {
            const h = (clamped / BAR_MAX) * 50
            return { top: 50 - h, height: h }
        }
        const h = (-clamped / BAR_MAX) * 50
        return { top: 50, height: h }
    }

    function barColor(cls: string): string {
        switch (cls) {
            case 'blunder':    return '#e53935'
            case 'mistake':    return '#ff7043'
            case 'inaccuracy': return '#ffb74d'
            case 'good':       return '#aed581'
            case 'excellent':  return '#8bc34a'
            default:           return '#5cb85c'
        }
    }

    function bgColor(cp: number | null): string {
        return (cp ?? 0) >= 0 ? '#1e1e3a' : '#0d0d1a'
    }
</script>

<div class="graph-wrap">
    <span class="axis-label top">+6</span>
    <span class="axis-label mid">0</span>
    <span class="axis-label bot">−6</span>

    <div class="bars">
        {#each moves as mv, i}
            {@const seg = whiteSegment(mv.score_cp)}
            <button
                class="bar-col"
                title="{mv.move_number}{mv.side === 'white' ? 'W' : 'B'} {mv.san}  {mv.score_cp !== null ? (mv.score_cp > 0 ? '+' : '') + mv.score_cp + 'cp' : 'M'}  (loss: {mv.loss_cp}cp)"
                onclick={() => onMoveClick?.(i)}
            >
                <div class="bar-bg" style="background:{bgColor(mv.score_cp)}"></div>
                <div
                    class="bar-fill"
                    style="top:{seg.top}%;height:{Math.max(seg.height, 1)}%;background:{barColor(mv.class)};"
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
    .bars { display: flex; align-items: stretch; height: 100%; gap: 1px; padding: 0 2px; }
    .bar-col { flex: 1; position: relative; background: none; border: none; cursor: pointer; padding: 0; min-width: 0; }
    .bar-col:hover .bar-fill { filter: brightness(1.4); }
    .bar-bg  { position: absolute; inset: 0; }
    .bar-fill { position: absolute; left: 0; right: 0; border-radius: 1px; transition: filter 0.1s; }
    .midline { position: absolute; top: 50%; left: 0; right: 0; height: 1px; background: rgba(255,255,255,0.15); pointer-events: none; }
    .axis-label { position: absolute; left: 5px; font-size: 9px; color: rgba(255,255,255,0.2); font-family: monospace; pointer-events: none; line-height: 1; }
    .axis-label.top { top: 4px; }
    .axis-label.mid { top: calc(50% - 5px); }
    .axis-label.bot { bottom: 4px; }
</style>
