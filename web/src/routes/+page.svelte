<script lang="ts">
    import { analyzeGame } from '$lib/api'
    import ScoreGraph from '$lib/components/ScoreGraph.svelte'
    import MoveTable from '$lib/components/MoveTable.svelte'
    import Board from '$lib/components/Board.svelte'
    import GameStats from '$lib/components/GameStats.svelte'
    import { analyzeFen } from '$lib/api'
    import FenPanel from '$lib/components/FenPanel.svelte'
    import type { AnalyzeResponse, GameResponse } from '$lib/types/analysis'

    let mode          = $state<'pgn' | 'fen'>('pgn')
    let pgn           = $state('')
    let depth         = $state(16)
    let loading       = $state(false)
    let error         = $state('')
    let response      = $state<AnalyzeResponse | null>(null)
    let gameIndex     = $state(0)
    let selectedIndex = $state<number | null>(null)

    const result       = $derived(response?.games[gameIndex] ?? null)
    const selectedMove = $derived(result && selectedIndex !== null ? result.moves[selectedIndex] : null)

    async function submit() {
        if (!pgn.trim()) return
        loading = true; error = ''; response = null; gameIndex = 0; selectedIndex = null
        try {
            response = await analyzeGame({ pgn, depth })
        } catch (e) {
            error = e instanceof Error ? e.message : String(e)
        } finally {
            loading = false
        }
    }

    function selectMove(index: number) { selectedIndex = index }
    function prevMove() { if (selectedIndex !== null && selectedIndex > 0) selectedIndex-- }
    function nextMove() { selectedIndex = selectedIndex === null ? 0 : Math.min(selectedIndex + 1, (result?.moves.length ?? 1) - 1) }
</script>

<svelte:head><title>Solance</title></svelte:head>

<svelte:window onkeydown={(e) => {
    if (!result) return
    if (e.key === 'ArrowRight') { nextMove(); e.preventDefault() }
    if (e.key === 'ArrowLeft')  { prevMove(); e.preventDefault() }
}} />

<main>
    {#if loading}<div class="progress-bar"></div>{/if}
    <div class="mode-tabs">
        <button class="tab {mode === 'pgn' ? 'active' : ''}" onclick={() => mode = 'pgn'}>Game Review</button>
        <button class="tab {mode === 'fen' ? 'active' : ''}" onclick={() => mode = 'fen'}>Position Analysis</button>
    </div>

    <header>
        <h1>Solance</h1>
        <p class="tagline">Local chess analysis. No cloud. No nonsense.</p>
    </header>

    {#if mode === 'pgn'}
    <section class="input-panel">
        <textarea bind:value={pgn} placeholder="Paste PGN here…" rows={5} disabled={loading}></textarea>
        <div class="controls">
            <label>
                <span>Depth</span>
                <input type="range" bind:value={depth} min={6} max={24} step={2} disabled={loading} />
                <span class="depth-val">{depth}</span>
            </label>
            <button onclick={submit} disabled={loading || !pgn.trim()}>
                {loading ? 'Analysing…' : 'Analyse'}
            </button>
        </div>
        {#if error}<p class="error">{error}</p>{/if}
    </section>

    {#if response && result}
        {#if response.games.length > 1}
            <div class="game-selector">
                {#each response.games as g, i}
                    <button
                        class="game-tab {i === gameIndex ? 'active' : ''}"
                        onclick={() => { gameIndex = i; selectedIndex = null }}
                    >
                        {g.white} vs {g.black}
                    </button>
                {/each}
            </div>
        {/if}

        <section class="result">
            <div class="game-header">
                <div class="players">
                    <span class="player white">{result.white}</span>
                    <span class="vs">vs</span>
                    <span class="player black">{result.black}</span>
                </div>
                <div class="meta">
                    <span>{result.opening ?? result.eco ?? result.event}</span>
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

            <div class="review-outer">
                <ScoreBar
                    scoreCp={selectedMove?.score_cp ?? null}
                    mateIn={null}
                />
                <div class="review-layout">
                <div class="left-panel">
                    <Board
                        fen={selectedMove?.fen_before ?? 'start'}
                        lastMove={selectedMove?.uci ?? null}
                        bestMove={selectedMove?.best_uci ?? null}
                        hasPrev={selectedIndex !== null && selectedIndex > 0}
                        hasNext={selectedIndex === null ? result.moves.length > 0 : selectedIndex < result.moves.length - 1}
                        onPrev={prevMove}
                        onNext={nextMove}
                    />
                    {#if selectedMove}
                        <div class="move-card">
                            <div class="move-card-top">
                                <span class="move-san">{selectedMove.san}</span>
                                <span class="move-class {selectedMove.class}">{selectedMove.class}</span>
                            </div>
                            <div class="move-stats">
                                <div class="stat"><span class="stat-label">Loss</span><span class="stat-val">{selectedMove.loss_cp}cp</span></div>
                                <div class="stat"><span class="stat-label">Score</span><span class="stat-val">{selectedMove.score_cp !== null ? (selectedMove.score_cp > 0 ? '+' : '') + selectedMove.score_cp : 'M'}</span></div>
                                <div class="stat"><span class="stat-label">Rank</span><span class="stat-val">{selectedMove.rank !== null ? '#' + selectedMove.rank : '—'}</span></div>
                                {#if selectedMove.best_uci && selectedMove.rank !== 1}
                                    <div class="stat"><span class="stat-label">Best</span><span class="stat-val mono">{selectedMove.best_uci}</span></div>
                                {/if}
                                <button class="copy-fen" onclick={() => navigator.clipboard.writeText(selectedMove?.fen_before ?? '')}>Copy FEN</button>
                            </div>
                        </div>
                    {:else}
                        <p class="select-hint">Click any move or bar to inspect</p>
                    {/if}
                </div>
                <div class="table-panel">
                    <MoveTable moves={result.moves} turningPoint={result.turning_point} selectedIndex={selectedIndex} onMoveClick={selectMove} />
                </div>
            </div>
            </div>
        </section>
    {/if}
{:else}
    <FenPanel />
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
    input[type=range] { accent-color: #5c5cf5; width: 100px; cursor: pointer; }
    .depth-val { font-family: monospace; color: rgba(255,255,255,0.6); font-size: 0.85rem; min-width: 2ch; }
    button { margin-left: auto; background: #5c5cf5; color: #fff; border: none; border-radius: 8px; padding: 0.55rem 1.75rem; font-size: 0.88rem; font-weight: 600; cursor: pointer; transition: background 0.15s, opacity 0.15s; }
    button:disabled { opacity: 0.4; cursor: not-allowed; }
    button:not(:disabled):hover { background: #4a4ae0; }
    .error { color: #e53935; font-size: 0.82rem; font-family: monospace; }

    .game-selector { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 1rem; }
    .game-tab { margin-left: 0; background: #10101e; border: 1px solid #1e1e36; border-radius: 6px; color: rgba(255,255,255,0.4); padding: 0.35rem 0.75rem; font-size: 0.78rem; cursor: pointer; }
    .game-tab.active { border-color: #5c5cf5; color: #fff; background: rgba(92,92,245,0.1); }
    .game-tab:hover { color: #fff; }

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

    .mode-tabs { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; }
    .tab { margin-left: 0; background: transparent; border: 1px solid #1e1e36; border-radius: 8px; color: rgba(255,255,255,0.4); padding: 0.45rem 1.1rem; font-size: 0.85rem; font-weight: 500; cursor: pointer; transition: all 0.15s; }
    .tab:hover { color: #fff; border-color: #3a3a5e; }
    .tab.active { background: rgba(92,92,245,0.12); border-color: #5c5cf5; color: #fff; }

    .progress-bar {
        position: fixed; top: 0; left: 0; right: 0; height: 3px; z-index: 100;
        background: linear-gradient(90deg, #5c5cf5, #8b5cf6, #5c5cf5);
        background-size: 200% 100%;
        animation: progress-slide 1.2s ease-in-out infinite;
    }
    @keyframes progress-slide {
        0%   { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }

    .mode-tabs { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; }
    .tab { margin-left: 0; background: transparent; border: 1px solid #1e1e36; border-radius: 8px; color: rgba(255,255,255,0.4); padding: 0.45rem 1.1rem; font-size: 0.85rem; font-weight: 500; cursor: pointer; transition: all 0.15s; }
    .tab:hover { color: #fff; border-color: #3a3a5e; }
    .tab.active { background: rgba(92,92,245,0.12); border-color: #5c5cf5; color: #fff; }

    .progress-bar {
        position: fixed; top: 0; left: 0; right: 0; height: 3px; z-index: 100;
        background: linear-gradient(90deg, #5c5cf5, #8b5cf6, #5c5cf5);
        background-size: 200% 100%;
        animation: progress-slide 1.2s ease-in-out infinite;
    }
    @keyframes progress-slide {
        0%   { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }

    .review-outer { display: flex; gap: 0.75rem; align-items: stretch; }
    .review-outer .review-layout { flex: 1; min-width: 0; }
    .copy-fen {
        margin-left: 0; margin-top: 0.5rem; width: 100%;
        background: #1a1a2e; border: 1px solid #2a2a4e;
        border-radius: 6px; color: rgba(255,255,255,0.5);
        padding: 0.35rem; font-size: 0.75rem; cursor: pointer;
        transition: all 0.15s;
    }
    .copy-fen:hover { background: #2a2a4e; color: #fff; }
</style>
