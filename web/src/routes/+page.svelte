<script lang="ts">
    import { analyzeGame } from '$lib/api'
    import ScoreGraph from '$lib/components/ScoreGraph.svelte'
    import GameStats from '$lib/components/GameStats.svelte'
    import MoveTable from '$lib/components/MoveTable.svelte'
    import Board from '$lib/components/Board.svelte'
    import type { AnalysisResponse } from '$lib/types/analysis'

    let pgn           = $state('')
    let depth         = $state(16)
    let loading       = $state(false)
    let error         = $state('')
    let result        = $state<AnalysisResponse | null>(null)
    let selectedIndex = $state<number | null>(null)

    const selectedMove = $derived(
        result && selectedIndex !== null ? result.moves[selectedIndex] : null
    )

    const boardFen = $derived(
        selectedMove ? selectedMove.fen_before ?? 'start' : 'start'
    )

    const boardOrientation = $derived(
        'white'
    )

    async function submit() {
        if (!pgn.trim()) return
        loading = true; error = ''; result = null; selectedIndex = null
        try {
            result = await analyzeGame({ pgn, depth })
        } catch (e) {
            error = e instanceof Error ? e.message : String(e)
        } finally {
            loading = false
        }
    }

    function selectMove(index: number) { selectedIndex = index }
</script>

<svelte:head><title>Solance</title></svelte:head>

<main>
    <header>
        <h1>Solance</h1>
        <p class="tagline">Local chess analysis. No cloud. No nonsense.</p>
    </header>

    <section class="input-panel">
        <textarea bind:value={pgn} placeholder="Paste PGN here…" rows={5} disabled={loading}></textarea>
        <div class="controls">
            <label>
                <span>Depth</span>
                <input type="number" bind:value={depth} min={6} max={24} disabled={loading} />
            </label>
            <button onclick={submit} disabled={loading || !pgn.trim()}>
                {loading ? 'Analysing…' : 'Analyse'}
            </button>
        </div>
        {#if error}<p class="error">{error}</p>{/if}
    </section>

    {#if result}
        <section class="result">
            <div class="game-header">
                <div class="players">
                    <span class="player white">{result.white}</span>
                    <span class="vs">vs</span>
                    <span class="player black">{result.black}</span>
                </div>
                <div class="meta">
                    <span>{result.event}</span>
                    <span class="result-badge">{result.result}</span>
                </div>
            </div>

            <div class="accuracy-row">
                <div class="acc-block">
                    <span class="acc-label">White accuracy</span>
                    <span class="acc-value">{result.white_accuracy.toFixed(1)}%</span>
                </div>
                <div class="acc-center">
                    <span class="engine-tag">{result.engine} · depth {result.depth}</span>
                </div>
                <div class="acc-block right">
                    <span class="acc-label">Black accuracy</span>
                    <span class="acc-value">{result.black_accuracy.toFixed(1)}%</span>
                </div>
            </div>

            {#if result.turning_point !== null}
                <p class="turning-note">
                    Turning point: move {Math.floor(result.turning_point / 2) + 1}{result.turning_point % 2 === 0 ? 'W' : 'B'}
                    — {result.moves[result.turning_point].san}
                    ({result.moves[result.turning_point].loss_cp}cp loss)
                </p>
            {/if}

            <GameStats moves={result.moves} />

            <ScoreGraph moves={result.moves} onMoveClick={selectMove} />

            <div class="review-layout">
                <div class="left-panel">
                    <div class="board-container">
                        <Board
                            fen={boardFen}
                            lastMove={selectedMove?.uci ?? null}
                            orientation={boardOrientation}
                        />
                    </div>
                    {#if selectedMove}
                        <div class="move-card">
                            <div class="move-card-top">
                                <span class="move-san">{selectedMove.san}</span>
                                <span class="move-class {selectedMove.class}">{selectedMove.class}</span>
                            </div>
                            <div class="move-stats">
                                <div class="stat">
                                    <span class="stat-label">Loss</span>
                                    <span class="stat-val">{selectedMove.loss_cp}cp</span>
                                </div>
                                <div class="stat">
                                    <span class="stat-label">Score</span>
                                    <span class="stat-val">{selectedMove.score_cp !== null ? (selectedMove.score_cp > 0 ? '+' : '') + selectedMove.score_cp : 'M'}</span>
                                </div>
                                <div class="stat">
                                    <span class="stat-label">Rank</span>
                                    <span class="stat-val">{selectedMove.rank !== null ? '#' + selectedMove.rank : '—'}</span>
                                </div>
                                {#if selectedMove.best_uci && selectedMove.rank !== 1}
                                    <div class="stat">
                                        <span class="stat-label">Best</span>
                                        <span class="stat-val mono">{selectedMove.best_uci}</span>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {:else}
                        <p class="select-hint">Click any move or bar to inspect</p>
                    {/if}
                </div>

                <div class="table-panel">
                    <MoveTable
                        moves={result.moves}
                        turningPoint={result.turning_point}
                        selectedIndex={selectedIndex}
                        onMoveClick={selectMove}
                    />
                </div>
            </div>
        </section>
    {/if}
</main>

<style>
    :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
    :global(body) { background: #0a0a18; color: #e0e0e0; font-family: 'Inter', system-ui, sans-serif; min-height: 100vh; }
    main { max-width: 960px; margin: 0 auto; padding: 2rem 1.5rem 4rem; }
    header { margin-bottom: 2rem; }
    h1 { font-size: 2rem; font-weight: 700; letter-spacing: -0.03em; color: #fff; }
    .tagline { color: rgba(255,255,255,0.3); font-size: 0.85rem; margin-top: 0.2rem; }

    .input-panel { display: flex; flex-direction: column; gap: 0.6rem; margin-bottom: 2rem; }
    textarea { width: 100%; background: #10101e; border: 1px solid #1e1e36; border-radius: 8px; color: #ddd; font-family: monospace; font-size: 0.82rem; padding: 0.75rem 1rem; resize: vertical; outline: none; transition: border-color 0.15s; line-height: 1.5; }
    textarea:focus { border-color: #5c5cf5; }
    .controls { display: flex; align-items: center; gap: 1rem; }
    label { display: flex; align-items: center; gap: 0.5rem; color: rgba(255,255,255,0.4); font-size: 0.82rem; }
    input[type=number] { width: 60px; background: #10101e; border: 1px solid #1e1e36; border-radius: 6px; color: #e0e0e0; padding: 0.35rem 0.6rem; font-size: 0.82rem; outline: none; }
    button { margin-left: auto; background: #5c5cf5; color: #fff; border: none; border-radius: 8px; padding: 0.55rem 1.75rem; font-size: 0.88rem; font-weight: 600; cursor: pointer; transition: background 0.15s, opacity 0.15s; }
    button:disabled { opacity: 0.4; cursor: not-allowed; }
    button:not(:disabled):hover { background: #4a4ae0; }
    .error { color: #e53935; font-size: 0.82rem; font-family: monospace; }

    .result { display: flex; flex-direction: column; gap: 1.25rem; }
    .game-header { display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; }
    .players { display: flex; align-items: center; gap: 0.6rem; font-size: 1.05rem; font-weight: 600; }
    .player.white { color: #f0f0f0; }
    .player.black { color: #999; }
    .vs { color: rgba(255,255,255,0.2); font-size: 0.75rem; }
    .meta { display: flex; gap: 0.6rem; color: rgba(255,255,255,0.3); font-size: 0.78rem; align-items: center; }
    .result-badge { background: #181830; border-radius: 4px; padding: 2px 7px; font-family: monospace; }

    .accuracy-row { display: flex; align-items: center; background: #10101e; border-radius: 10px; padding: 1rem 1.5rem; border: 1px solid #1e1e36; }
    .acc-block { display: flex; flex-direction: column; gap: 0.15rem; flex: 1; }
    .acc-block.right { align-items: flex-end; }
    .acc-label { font-size: 0.68rem; color: rgba(255,255,255,0.25); text-transform: uppercase; letter-spacing: 0.08em; }
    .acc-value { font-size: 1.6rem; font-weight: 700; color: #fff; letter-spacing: -0.02em; }
    .acc-center { flex: 1; text-align: center; }
    .engine-tag { font-size: 0.72rem; color: rgba(255,255,255,0.2); font-family: monospace; }

    .turning-note { font-size: 0.8rem; color: rgba(229,57,53,0.75); font-family: monospace; padding: 0.4rem 0.75rem; background: rgba(229,57,53,0.05); border-left: 2px solid rgba(229,57,53,0.35); border-radius: 0 4px 4px 0; }

    .review-layout { display: grid; grid-template-columns: 280px 1fr; gap: 1.25rem; align-items: start; }

    .left-panel { display: flex; flex-direction: column; gap: 0.75rem; position: sticky; top: 1rem; }
    .board-container { width: 100%; border-radius: 8px; overflow: hidden; border: 1px solid #1e1e36; }

    .move-card { background: #10101e; border: 1px solid #1e1e36; border-radius: 10px; padding: 0.9rem 1rem; }
    .move-card-top { display: flex; align-items: baseline; justify-content: space-between; margin-bottom: 0.6rem; }
    .move-san { font-size: 1.3rem; font-weight: 700; color: #fff; font-family: monospace; }
    .move-class { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.06em; font-weight: 600; padding: 2px 7px; border-radius: 4px; }
    .move-class.best       { color: #5cb85c; background: rgba(92,184,92,0.1); }
    .move-class.excellent  { color: #8bc34a; background: rgba(139,195,74,0.1); }
    .move-class.good       { color: #aaa;    background: rgba(255,255,255,0.06); }
    .move-class.inaccuracy { color: #ffb74d; background: rgba(255,183,77,0.1); }
    .move-class.mistake    { color: #ff7043; background: rgba(255,112,67,0.1); }
    .move-class.blunder    { color: #e53935; background: rgba(229,57,53,0.1); }
    .move-stats { display: flex; flex-direction: column; gap: 0.35rem; }
    .stat { display: flex; justify-content: space-between; }
    .stat-label { font-size: 0.72rem; color: rgba(255,255,255,0.3); }
    .stat-val { font-size: 0.82rem; color: #ddd; font-family: monospace; }
    .stat-val.mono { font-size: 0.75rem; }

    .select-hint { font-size: 0.78rem; color: rgba(255,255,255,0.2); text-align: center; padding: 1.5rem 1rem; background: #10101e; border: 1px solid #1e1e36; border-radius: 10px; font-family: monospace; }
    .table-panel { min-width: 0; }
</style>

<svelte:window onkeydown={(e) => {
    if (!result) return
    if (e.key === 'ArrowRight') {
        selectedIndex = Math.min((selectedIndex ?? -1) + 1, result.moves.length - 1)
        e.preventDefault()
    }
    if (e.key === 'ArrowLeft') {
        const next = (selectedIndex ?? 0) - 1
        selectedIndex = next < 0 ? null : next
        e.preventDefault()
    }
}} />
