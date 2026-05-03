export type Classification = 'best' | 'excellent' | 'good' | 'inaccuracy' | 'mistake' | 'blunder'

export interface MoveResponse {
    move_number:      number
    side:             'white' | 'black'
    san:              string
    uci:              string
    fen_before:       string
    best_uci:         string | null
    /** White-relative centipawns. ±1500 used for forced mate positions. */
    score_cp:         number | null
    loss_cp:          number
    win_percent_loss: number
    rank:             number | null
    class:            Classification
    decided:          boolean
}

export interface AnalysisResponse {
    event:          string
    white:          string
    black:          string
    result:         string
    engine:         string
    depth:          number
    white_accuracy: number
    black_accuracy: number
    turning_point:  number | null
    moves:          MoveResponse[]
}

export interface AnalyzeRequest {
    pgn:     string
    depth?:  number
    engine?: string
}
