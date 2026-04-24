<script lang="ts">
    import type { MoveResponse } from '../types/analysis'

    const { moves, turningPoint, selectedIndex, onMoveClick }: {
        moves:          MoveResponse[]
        turningPoint:   number | null
        selectedIndex:  number | null
        onMoveClick?:   (index: number) => void
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
        good:       '#888',
        inaccuracy: '#ffb74d',
        mistake:    '#ff7043',
        blunder:    '#e53935',
    }

    type Pair = [MoveResponse, MoveResponse | null, number, number | null]

    const pairs = $derived(
        moves.reduce<Pair[]>((acc, mv, i) => {
            if (i % 2 === 0) acc.push([mv, null, i, null])
            else {
                acc[acc.length - 1][1] = mv
                acc[acc.length - 1][3] = i
            }
            return acc
        }, [])
    )
</script>

<div class="table-wrap">
    <table>
        <thead>
            <tr>
                <th class="col-num">#</th>
                <th class="col-move">White</th>
                <th class="col-loss">cp</th>
                <th class="col-move">Black</th>
                <th class="col-loss">cp</th>
            </tr>
        </thead>
        <tbody>
            {#each pairs as [w, b, wi, bi], i}
                <tr class:turning={turningPoint !== null && (wi === turningPoint || bi === turningPoint)}>
                    <td class="col-num">{w.move_number}</td>

                    <td
                        class="col-move"
                        class:selected={selectedIndex === wi}
                        role="button"
                        tabindex="0"
                        onclick={() => onMoveClick?.(wi)}
                        onkeydown={(e) => e.key === 'Enter' && onMoveClick?.(wi)}
                    >
                        <span class="icon" style="color:{CLASS_COLOR[w.class]}">{CLASS_ICON[w.class]}</span>
                        <span class="san">{w.san}</span>
                        {#if w.rank !== null && w.rank !== 1}
                            <span class="rank">#{w.rank}</span>
                        {/if}
                    </td>
                    <td class="col-loss" style="color:{CLASS_COLOR[w.class]}">{w.loss_cp}</td>

                    {#if b && bi !== null}
                        <td
                            class="col-move"
                            class:selected={selectedIndex === bi}
                            role="button"
                            tabindex="0"
                            onclick={() => onMoveClick?.(bi!)}
                            onkeydown={(e) => e.key === 'Enter' && onMoveClick?.(bi!)}
                        >
                            <span class="icon" style="color:{CLASS_COLOR[b.class]}">{CLASS_ICON[b.class]}</span>
                            <span class="san">{b.san}</span>
                            {#if b.rank !== null && b.rank !== 1}
                                <span class="rank">#{b.rank}</span>
                            {/if}
                        </td>
                        <td class="col-loss" style="color:{CLASS_COLOR[b.class]}">{b.loss_cp}</td>
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
        max-height: 400px;
        border-radius: 8px;
        border: 1px solid #1e1e36;
        scroll-padding-top: 32px;
    }
    table {
        width: 100%;
        border-collapse: collapse;
        font-family: monospace;
        font-size: 0.85rem;
        table-layout: fixed;
    }
    thead th {
        position: sticky;
        top: 0;
        z-index: 1;
        background: #0f0f1e;
        color: rgba(255,255,255,0.3);
        font-weight: 500;
        text-align: left;
        padding: 7px 10px;
        font-size: 0.7rem;
        text-transform: uppercase;
        letter-spacing: 0.07em;
        border-bottom: 1px solid #1e1e36;
    }
    tbody tr { border-bottom: 1px solid #181828; }
    tbody tr:hover { background: #181830; }
    tbody tr.turning { background: rgba(229,57,53,0.07); }

    td { padding: 5px 10px; color: #ccc; white-space: nowrap; overflow: hidden; }

    .col-num  { width: 2.5rem; color: rgba(255,255,255,0.2); font-size: 0.75rem; }
    .col-move { cursor: pointer; }
    .col-move:hover { color: #fff; }
    .col-move.selected { background: rgba(92,92,245,0.15); color: #fff; }
    .col-loss { width: 3rem; text-align: right; font-size: 0.78rem; }

    .icon { font-size: 0.7rem; margin-right: 5px; }
    .san  { }
    .rank { font-size: 0.68rem; color: rgba(255,255,255,0.25); margin-left: 4px; }
</style>
