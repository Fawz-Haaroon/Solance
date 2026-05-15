import type { AnalyzeRequest, AnalyzeResponse } from './types/analysis'

const BASE = 'http://localhost:4242'

export async function analyzeGame(req: AnalyzeRequest): Promise<AnalyzeResponse> {
    const res = await fetch(`${BASE}/analyze`, {
        method:  'POST',
        headers: { 'Content-Type': 'application/json' },
        body:    JSON.stringify(req),
    })
    if (!res.ok) throw new Error(`analysis failed (${res.status}): ${await res.text()}`)
    return res.json()
}

export async function analyzeFen(fen: string, depth: number): Promise<FenEvalResult> {
    const res = await fetch(`${BASE}/analyze/fen`, {
        method:  'POST',
        headers: { 'Content-Type': 'application/json' },
        body:    JSON.stringify({ fen, depth }),
    })
    if (!res.ok) throw new Error(`fen analysis failed (${res.status}): ${await res.text()}`)
    return res.json()
}

export interface FenEvalResult {
    fen:       string
    depth:     number
    score_cp:  number | null
    mate_in:   number | null
    best_move: string | null
    top_moves: {
        mv:       string
        score_cp: number | null
        mate_in:  number | null
        rank:     number
        pv:       string[]
    }[]
}
