<script lang="ts">
    import { analyzeFen, type FenEvalResult } from '$lib/api'
    import Board from './Board.svelte'

    let fen     = $state('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')
    let depth   = $state(18)
    let loading = $state(false)
    let error   = $state('')
    let result  = $state<FenEvalResult | null>(null)

    async function submit() {
        if (!fen.trim()) return
        loading = true; error = ''; result = null
        try {
            result = await analyzeFen(fen.trim(), depth)
        } catch(e) {
            error = e instanceof Error ? e.message : String(e)
        } finally {
            loading = false
        }
    }

    function formatScore(cp: number | null, mate: number | null): string {
        if (mate !== null) return mate > 0 ? `M${mate}` : `-M${Math.abs(mate)}`
        if (cp  !== null) return (cp > 0 ? '+' : '') + (cp / 100).toFixed(2)
        return '?'
    }
</script>

<div class="fen-panel">
    <div class="fen-input-row">
        <input
            bind:value={fen}
            placeholder="FEN string…"
            disabled={loading}
            onkeydown={(e) => e.key === 'Enter' && submit()}
        />
        <label class="depth-label">
            <span>Depth</span>
            <input type="range" bind:value={depth} min={6} max={24} step={2} disabled={loading} />
            <span class="dv">{depth}</span>
        </label>
        <button onclick={submit} disabled={loading || !fen.trim()}>
            {loading ? 'Evaluating…' : 'Evaluate'}
        </button>
    </div>
    {#if error}<p class="error">{error}</p>{/if}

    {#if result}
        <div class="fen-result">
            <div class="fen-layout">
                <div class="fen-board">
                    <Board fen={result.fen} lastMove={result.best_move} bestMove={result.best_move} />
                </div>
                <div class="fen-info">
                    <div class="score-big">{formatScore(result.score_cp, result.mate_in)}</div>
                    <p class="score-sub">depth {result.depth} · {result.score_cp !== null ? 'cp' : 'forced mate'}</p>
                    {#if result.best_move}
                        <div class="best-move">Best: <span>{result.best_move}</span></div>
                    {/if}
                    <div class="top-moves">
                        <p class="top-label">Top moves</p>
                        {#each result.top_moves as m}
                            <div class="top-row">
                                <span class="top-rank">#{m.rank}</span>
                                <span class="top-mv">{m.mv}</span>
                                <span class="top-score">{formatScore(m.score_cp, m.mate_in)}</span>
                                {#if m.pv.length > 1}
                                    <span class="top-pv">{m.pv.slice(1, 4).join(' ')}</span>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </div>
            </div>
        </div>
    {/if}
</div>

<style>
    .fen-panel { display: flex; flex-direction: column; gap: 1rem; }
    .fen-input-row { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
    .fen-input-row input {
        flex: 1;
        min-width: 200px;
        background: #10101e;
        border: 1px solid #1e1e36;
        border-radius: 8px;
        color: #ddd;
        font-family: monospace;
        font-size: 0.82rem;
        padding: 0.55rem 0.9rem;
        outline: none;
        transition: border-color 0.15s;
    }
    input:focus { border-color: #5c5cf5; }
    .depth-label { display: flex; align-items: center; gap: 0.4rem; color: rgba(255,255,255,0.4); font-size: 0.8rem; white-space: nowrap; }
    input[type=range] { accent-color: #5c5cf5; width: 80px; }
    .dv { font-family: monospace; color: rgba(255,255,255,0.6); min-width: 2ch; }
    button { background: #5c5cf5; color: #fff; border: none; border-radius: 8px; padding: 0.55rem 1.25rem; font-size: 0.88rem; font-weight: 600; cursor: pointer; white-space: nowrap; }
    button:disabled { opacity: 0.4; cursor: not-allowed; }
    button:not(:disabled):hover { background: #4a4ae0; }
    .error { color: #e53935; font-size: 0.82rem; font-family: monospace; }

    .fen-layout { display: grid; grid-template-columns: 280px 1fr; gap: 1.5rem; align-items: start; }
    .fen-board { width: 100%; }
    .fen-info { display: flex; flex-direction: column; gap: 0.75rem; }
    .score-big { font-size: 2.5rem; font-weight: 700; color: #fff; letter-spacing: -0.03em; font-family: monospace; }
    .score-sub { font-size: 0.75rem; color: rgba(255,255,255,0.3); margin-top: -0.5rem; }
    .best-move { font-size: 0.9rem; color: rgba(255,255,255,0.5); font-family: monospace; }
    .best-move span { color: #5cb85c; font-weight: 600; }

    .top-moves { display: flex; flex-direction: column; gap: 0.3rem; }
    .top-label { font-size: 0.68rem; text-transform: uppercase; letter-spacing: 0.08em; color: rgba(255,255,255,0.25); margin-bottom: 0.2rem; }
    .top-row { display: flex; align-items: center; gap: 0.75rem; font-family: monospace; font-size: 0.82rem; padding: 0.3rem 0.6rem; background: #10101e; border-radius: 5px; border: 1px solid #1e1e36; }
    .top-rank { color: rgba(255,255,255,0.25); width: 2ch; }
    .top-mv   { color: #fff; font-weight: 600; min-width: 5ch; }
    .top-score { color: #5cb85c; min-width: 6ch; }
    .top-pv   { color: rgba(255,255,255,0.3); font-size: 0.75rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
