<script lang="ts">
    import type { MoveResponse, Classification } from '../types/analysis'

    const { moves }: { moves: MoveResponse[] } = $props()

    type Counts = Record<Classification, number>
    function count(side: 'white' | 'black'): Counts {
        const c = { best:0, excellent:0, good:0, inaccuracy:0, mistake:0, blunder:0 } as Counts
        moves.filter(m => m.side === side).forEach(m => c[m.class]++)
        return c
    }

    const white = $derived(count('white'))
    const black = $derived(count('black'))

    const rows: { label: string; key: Classification; color: string }[] = [
        { label: 'Best',       key: 'best',       color: '#5cb85c' },
        { label: 'Excellent',  key: 'excellent',  color: '#8bc34a' },
        { label: 'Good',       key: 'good',       color: '#aaa'    },
        { label: 'Inaccuracy', key: 'inaccuracy', color: '#ffb74d' },
        { label: 'Mistake',    key: 'mistake',    color: '#ff7043' },
        { label: 'Blunder',    key: 'blunder',    color: '#e53935' },
    ]
</script>

<div class="stats">
    <div class="col white-col">
        {#each rows as row}
            <span class="val" style="color:{row.color}">{white[row.key]}</span>
        {/each}
    </div>
    <div class="labels">
        {#each rows as row}
            <span class="lbl">{row.label}</span>
        {/each}
    </div>
    <div class="col black-col">
        {#each rows as row}
            <span class="val" style="color:{row.color}">{black[row.key]}</span>
        {/each}
    </div>
</div>

<style>
    .stats {
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        gap: 0;
        background: #10101e;
        border: 1px solid #1e1e36;
        border-radius: 8px;
        padding: 0.5rem 1rem;
    }
    .col { display: flex; flex-direction: column; gap: 0.25rem; }
    .white-col { align-items: flex-end; }
    .black-col { align-items: flex-start; }
    .val { font-size: 0.9rem; font-weight: 600; font-family: monospace; min-height: 1.4rem; display: flex; align-items: center; }
    .labels { display: flex; flex-direction: column; gap: 0.25rem; align-items: center; padding: 0 1rem; }
    .lbl { font-size: 0.7rem; color: rgba(255,255,255,0.3); text-transform: uppercase; letter-spacing: 0.05em; min-height: 1.4rem; display: flex; align-items: center; }
</style>
