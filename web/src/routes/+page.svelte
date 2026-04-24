<script lang="ts">
    import { analyzeGame } from '$lib/api'
    import ScoreGraph from '$lib/components/ScoreGraph.svelte'
    import MoveTable from '$lib/components/MoveTable.svelte'
    import type { AnalysisResponse } from '$lib/types/analysis'

    let pgn     = $state('')
    let depth   = $state(16)
    let loading = $state(false)
    let error   = $state('')
    let result  = $state<AnalysisResponse | null>(null)

    async function submit() {
        if (!pgn.trim()) return
        loading = true
        error   = ''
        result  = null
        try {
            result = await analyzeGame({ pgn, depth })
        } catch (e) {
            error = e instanceof Error ? e.message : String(e)
        } finally {
            loading = false
        }
    }
</script>

<svelte:head><title>Solance</title></svelte:head>

<main>
    <header>
        <h1>Solance</h1>
        <p class="tagline">Local chess analysis. No cloud. No nonsense.</p>
    </header>

    <section class="input-panel">
        <textarea
            bind:value={pgn}
            placeholder="Paste PGN here…"
            rows={6}
            disabled={loading}
        ></textarea>
        <div class="controls">
            <label>
                <span>Depth</span>
                <input type="number" bind:value={depth} min={6} max={24} disabled={loading} />
            </label>
            <button onclick={submit} disabled={loading || !pgn.trim()}>
                {loading ? 'Analysing…' : 'Analyse'}
            </button>
        </div>
        {#if error}
            <p class="error">{error}</p>
        {/if}
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
                    <span class="acc-label">White</span>
                    <span class="acc-value">{result.white_accuracy.toFixed(1)}%</span>
                </div>
                <div class="acc-center">
                    <span class="engine-tag">{result.engine} · depth {result.depth}</span>
                </div>
                <div class="acc-block right">
                    <span class="acc-label">Black</span>
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

            <ScoreGraph moves={result.moves} />
            <MoveTable moves={result.moves} turningPoint={result.turning_point} />
        </section>
    {/if}
</main>

<style>
    :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
    :global(body) {
        background: #0d0d1a;
        color: #e0e0e0;
        font-family: 'Inter', system-ui, sans-serif;
        min-height: 100vh;
    }
    main { max-width: 860px; margin: 0 auto; padding: 2rem 1.5rem; }
    header { margin-bottom: 2.5rem; }
    h1 { font-size: 2.25rem; font-weight: 700; letter-spacing: -0.03em; color: #fff; }
    .tagline { color: rgba(255,255,255,0.35); font-size: 0.9rem; margin-top: 0.25rem; }

    .input-panel { display: flex; flex-direction: column; gap: 0.75rem; margin-bottom: 2rem; }
    textarea {
        width: 100%;
        background: #12122a;
        border: 1px solid #2a2a3e;
        border-radius: 8px;
        color: #e0e0e0;
        font-family: monospace;
        font-size: 0.85rem;
        padding: 0.75rem 1rem;
        resize: vertical;
        outline: none;
        transition: border-color 0.15s;
    }
    textarea:focus { border-color: #5c5cf5; }
    .controls { display: flex; align-items: center; gap: 1rem; }
    label { display: flex; align-items: center; gap: 0.5rem; color: rgba(255,255,255,0.5); font-size: 0.85rem; }
    input[type=number] {
        width: 64px;
        background: #12122a;
        border: 1px solid #2a2a3e;
        border-radius: 6px;
        color: #e0e0e0;
        padding: 0.4rem 0.6rem;
        font-size: 0.85rem;
        outline: none;
    }
    button {
        margin-left: auto;
        background: #5c5cf5;
        color: #fff;
        border: none;
        border-radius: 8px;
        padding: 0.6rem 1.75rem;
        font-size: 0.9rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s, opacity 0.15s;
    }
    button:disabled { opacity: 0.45; cursor: not-allowed; }
    button:not(:disabled):hover { background: #4a4ae0; }
    .error { color: #e53935; font-size: 0.85rem; font-family: monospace; }

    .result { display: flex; flex-direction: column; gap: 1rem; }
    .game-header { display: flex; justify-content: space-between; align-items: baseline; flex-wrap: wrap; gap: 0.5rem; }
    .players { display: flex; align-items: center; gap: 0.75rem; font-size: 1.1rem; font-weight: 600; }
    .player.white { color: #f5f5f5; }
    .player.black { color: #aaa; }
    .vs { color: rgba(255,255,255,0.25); font-size: 0.8rem; font-weight: 400; }
    .meta { display: flex; gap: 0.75rem; color: rgba(255,255,255,0.35); font-size: 0.8rem; }
    .result-badge { background: #1e1e32; border-radius: 4px; padding: 1px 6px; color: rgba(255,255,255,0.5); }

    .accuracy-row {
        display: flex;
        align-items: center;
        background: #12122a;
        border-radius: 8px;
        padding: 0.85rem 1.25rem;
        border: 1px solid #2a2a3e;
    }
    .acc-block { display: flex; flex-direction: column; gap: 0.1rem; flex: 1; }
    .acc-block.right { align-items: flex-end; }
    .acc-label { font-size: 0.7rem; color: rgba(255,255,255,0.3); text-transform: uppercase; letter-spacing: 0.06em; }
    .acc-value { font-size: 1.5rem; font-weight: 700; color: #fff; }
    .acc-center { flex: 1; text-align: center; }
    .engine-tag { font-size: 0.75rem; color: rgba(255,255,255,0.25); font-family: monospace; }

    .turning-note {
        font-size: 0.82rem;
        color: rgba(229,57,53,0.8);
        font-family: monospace;
        padding: 0.4rem 0.75rem;
        background: rgba(229,57,53,0.06);
        border-left: 2px solid rgba(229,57,53,0.4);
        border-radius: 0 4px 4px 0;
    }
</style>
