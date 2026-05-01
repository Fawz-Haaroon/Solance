<script lang="ts">
    import { onMount, onDestroy } from 'svelte'

    const { fen, lastMove, orientation = 'white' }: {
        fen:          string
        lastMove:     string | null
        orientation?: 'white' | 'black'
    } = $props()

    let container: HTMLDivElement
    let board: any = null

    onMount(async () => {
        const { Chessboard, COLOR } = await import('cm-chessboard')
        board = new Chessboard(container, {
            position:    fen || 'start',
            orientation: orientation === 'white' ? COLOR.white : COLOR.black,
            assetsUrl:   '/',
        })
    })

    onDestroy(() => { board?.destroy?.(); board = null })

    $effect(() => {
        if (!board || !fen) return
        board.setPosition(fen, true)
    })
</script>

<svelte:head>
    <link rel="stylesheet" href="/chessboard.css" />
</svelte:head>

<div bind:this={container} class="board"></div>

<style>
    .board { width: 100%; aspect-ratio: 1; }
</style>
