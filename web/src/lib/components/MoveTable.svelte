<script lang="ts">
    import type { MoveResponse } from '../types/analysis'

    const { moves, turningPoint }: {
        moves:         MoveResponse[]
        turningPoint:  number | null
    } = $props()

    const CLASS_ICON: Record<string, string> = {
        best:       '✦',
        excellent:  '✓',
        good:       '·',
        inaccuracy: '?',
        mistake:    '?!',
        blunder:    '??',
    }

    const CLASS_COLOR: Record<string, string> = {
        best:       '#5cb85c',
        excellent:  '#8bc34a',
        good:       '#ccc',
        inaccuracy: '#ffb74d',
        mistake:    '#ff7043',
        blunder:    '#e53935',
    }

    type Pair = [MoveResponse, MoveResponse | null]

    const pairs = $derived(
        moves.reduce<Pair[]>((acc, mv, i) => {
            if (i % 2 === 0) acc.push([mv, null])
            else acc[acc.length - 1][1] = mv
            return acc
        }, [])
    )
</script>

<div class="table-wrap">
    <table>
        <thead>
            <tr>
                <th>#</th>
                <th>White</th>
                <th>loss</th>
                <th>Black</th>
                <th>loss</th>
            </tr>
        </thead>
        <tbody>
            {#each pairs as [w, b], i}
                <tr class:turning={turningPoint !== null && (i * 2 === turningPoint || i * 2 + 1 === turningPoint)}>
                    <td class="num">{w.move_number}</td>
                    <td class="move">
                        <span class="icon" style="color: {CLASS_COLOR[w.class]}">{CLASS_ICON[w.class]}</span>
                        {w.san}
                        {#if w.rank !== null && w.rank !== 1}
                            <span class="rank">#{w.rank}</span>
                        {/if}
                    </td>
                    <td class="loss" style="color: {CLASS_COLOR[w.class]}">{w.loss_cp}</td>
                    {#if b}
                        <td class="move">
                            <span class="icon" style="color: {CLASS_COLOR[b.class]}">{CLASS_ICON[b.class]}</span>
                            {b.san}
                            {#if b.rank !== null && b.rank !== 1}
                                <span class="rank">#{b.rank}</span>
                            {/if}
                        </td>
                        <td class="loss" style="color: {CLASS_COLOR[b.class]}">{b.loss_cp}</td>
                    {:else}
                        <td></td><td></td>
                    {/if}
                </tr>
            {/each}
        </tbody>
    </table>
</div>

<style>
    .table-wrap {
        overflow-y: auto;
        max-height: 420px;
        border-radius: 6px;
        border: 1px solid #2a2a3e;
    }
    table {
        width: 100%;
        border-collapse: collapse;
        font-family: monospace;
        font-size: 0.875rem;
    }
    thead th {
        position: sticky;
        top: 0;
        background: #12122a;
        color: rgba(255,255,255,0.4);
        font-weight: 500;
        text-align: left;
        padding: 6px 10px;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }
    tbody tr { border-bottom: 1px solid #1e1e32; }
    tbody tr:hover { background: #1e1e32; }
    tbody tr.turning { background: rgba(229,57,53,0.08); }
    td { padding: 5px 10px; color: #ddd; white-space: nowrap; }
    td.num  { color: rgba(255,255,255,0.25); width: 2rem; }
    td.loss { width: 3rem; font-size: 0.8rem; }
    td.move { min-width: 90px; }
    .icon   { font-size: 0.75rem; margin-right: 4px; }
    .rank   { font-size: 0.7rem; color: rgba(255,255,255,0.3); margin-left: 4px; }
</style>
