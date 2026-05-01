<script lang="ts">
    const { fen, orientation = 'white' }: {
        fen:          string
        orientation?: 'white' | 'black'
    } = $props()

    const UNICODE: Record<string, string> = {
        K:'♔', Q:'♕', R:'♖', B:'♗', N:'♘', P:'♙',
        k:'♚', q:'♛', r:'♜', b:'♝', n:'♞', p:'♟',
    }

    const START = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
    const FILES  = ['a','b','c','d','e','f','g','h']
    const RANKS  = ['8','7','6','5','4','3','2','1']

    function parseFen(f: string): (string|null)[][] {
        const src  = (!f || f === 'start') ? START : f
        const grid = Array.from({length:8}, () => Array<string|null>(8).fill(null))
        src.split(' ')[0].split('/').forEach((row, r) => {
            let c = 0
            for (const ch of row) {
                if (/\d/.test(ch)) c += +ch
                else { grid[r][c] = ch; c++ }
            }
        })
        return grid
    }

    const grid = $derived(parseFen(fen))
    const rows = $derived(orientation === 'white' ? [0,1,2,3,4,5,6,7] : [7,6,5,4,3,2,1,0])
    const cols = $derived(orientation === 'white' ? [0,1,2,3,4,5,6,7] : [7,6,5,4,3,2,1,0])
</script>

<div class="board">
    <div class="inner">
        {#each rows as r}
            <div class="rank-label">{RANKS[r]}</div>
            {#each cols as c}
                {@const p = grid[r][c]}
                {@const light = (r + c) % 2 === 0}
                <div class="sq {light ? 'lt' : 'dk'}">
                    {#if p}
                        <span class="pc {p === p.toUpperCase() ? 'wp' : 'bp'}">{UNICODE[p]}</span>
                    {/if}
                </div>
            {/each}
        {/each}
        <div class="corner"></div>
        {#each cols as c}
            <div class="file-label">{FILES[c]}</div>
        {/each}
    </div>
</div>

<style>
    .board {
        width: 100%;
        aspect-ratio: 1;
        container-type: inline-size;
    }
    .inner {
        width: 100%;
        height: 100%;
        display: grid;
        grid-template-columns: 16px repeat(8, 1fr);
        grid-template-rows: repeat(8, 1fr) 16px;
        font-family: 'Segoe UI Symbol', 'Apple Color Emoji', serif;
    }
    .sq {
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }
    .lt { background: #f0d9b5; }
    .dk { background: #b58863; }
    .pc {
        font-size: 10cqi;
        line-height: 1;
        user-select: none;
        display: block;
    }
    .wp { color: #fff; filter: drop-shadow(0 1px 2px rgba(0,0,0,0.9)) drop-shadow(0 0 1px rgba(0,0,0,0.9)); }
    .bp { color: #111; filter: drop-shadow(0 1px 0 rgba(255,255,255,0.6)); }
    .rank-label, .file-label, .corner {
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 9px;
        color: rgba(255,255,255,0.35);
        background: #1a1a2e;
        font-family: monospace;
    }
    .rank-label { grid-column: 1; }
    .file-label { grid-row: 9; }
    .corner     { grid-column: 1; grid-row: 9; }
</style>
