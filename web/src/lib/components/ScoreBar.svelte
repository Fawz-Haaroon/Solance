<script lang="ts">
    const { scoreCp, mateIn }: {
        scoreCp: number | null
        mateIn:  number | null
    } = $props()

    const MAX_CP = 600

    const whiteShare = $derived((() => {
        if (mateIn !== null) return mateIn > 0 ? 1.0 : 0.0
        if (scoreCp === null) return 0.5
        const clamped = Math.max(-MAX_CP, Math.min(MAX_CP, scoreCp))
        return 0.5 + clamped / (MAX_CP * 2)
    })())

    const label = $derived((() => {
        if (mateIn !== null) return mateIn > 0 ? \`M\${mateIn}\` : \`M\${Math.abs(mateIn)}\`
        if (scoreCp === null) return '0.00'
        const abs = Math.abs(scoreCp / 100)
        return (scoreCp >= 0 ? '+' : '-') + abs.toFixed(2)
    })())

    const labelOnBlack = $derived(whiteShare > 0.5)
</script>

<div class="bar-wrap">
    <div class="bar">
        <div class="white-seg" style="height: {whiteShare * 100}%"></div>
        <div class="black-seg"></div>
    </div>
    <span class="label {labelOnBlack ? 'on-black' : 'on-white'}">{label}</span>
</div>

<style>
    .bar-wrap {
        position: relative;
        width: 28px;
        height: 100%;
        min-height: 280px;
        display: flex;
        flex-direction: column;
        align-items: center;
        border-radius: 4px;
        overflow: hidden;
        border: 1px solid #1e1e36;
        flex-shrink: 0;
    }
    .bar { width: 100%; height: 100%; display: flex; flex-direction: column; }
    .white-seg { background: #f0f0f0; transition: height 0.3s ease; }
    .black-seg { background: #1a1a1a; flex: 1; }
    .label {
        position: absolute;
        font-size: 9px;
        font-family: monospace;
        font-weight: 600;
        writing-mode: vertical-rl;
        transform: rotate(180deg);
        pointer-events: none;
        letter-spacing: 0.05em;
    }
    .label.on-black { top: 6px; color: rgba(255,255,255,0.7); }
    .label.on-white { bottom: 6px; color: rgba(0,0,0,0.5); }
</style>
