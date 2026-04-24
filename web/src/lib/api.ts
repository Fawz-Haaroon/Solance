import type { AnalyzeRequest, AnalysisResponse } from './types/analysis'

const BASE = 'http://localhost:4242'

export async function analyzeGame(req: AnalyzeRequest): Promise<AnalysisResponse> {
    const res = await fetch(`${BASE}/analyze`, {
        method:  'POST',
        headers: { 'Content-Type': 'application/json' },
        body:    JSON.stringify(req),
    })

    if (!res.ok) {
        const msg = await res.text()
        throw new Error(`analysis failed (${res.status}): ${msg}`)
    }

    return res.json()
}
