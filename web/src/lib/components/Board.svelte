<script lang="ts">
    import { onMount } from 'svelte'

    const { fen, orientation = 'white' }: {
        fen:          string
        orientation?: 'white' | 'black'
    } = $props()

    let container: HTMLDivElement
    let board: { setPosition: (f: string, a: boolean) => void; destroy: () => void } | null = null

    onMount(async () => {
        const { Chessboard, COLOR, PIECES_FILE_TYPE } = await import('cm-chessboard')
        board = new Chessboard(container, {
            position:    fen || 'start',
            orientation: orientation === 'white' ? COLOR.white : COLOR.black,
            assetsCache: false,
            style: {
                pieces: {
                    type:     PIECES_FILE_TYPE.svgSprite,
                    file:     '/pieces/standard.svg',
                    tileSize: 40,
                }
            }
        }) as typeof board

        return () => { board?.destroy(); board = null }
    })

    $effect(() => {
        board?.setPosition(fen || 'start', true)
    })
</script>

<svelte:head>
    <link rel="stylesheet" href="/chessboard.css" />
</svelte:head>

<div bind:this={container} class="board"></div>

<style>
    .board { width: 100%; aspect-ratio: 1; }
</style>
