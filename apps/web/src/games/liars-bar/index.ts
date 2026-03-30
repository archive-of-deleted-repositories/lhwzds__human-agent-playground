import type { GameSession } from '@human-agent-playground/core'

import type { GameModule, GameSummary } from '../types'
import { LiarsBarWorkspace } from './LiarsBarWorkspace'

interface LiarsBarPublicState {
  kind: 'liars-bar'
  turn: string
  status: string
  winner: string | null
}

function getLiarsBarSummary(session: GameSession): GameSummary {
  const state = session.state as LiarsBarPublicState

  return {
    turn: state.turn,
    status: state.status,
    winner: state.status === 'finished' ? state.winner ?? 'draw' : state.winner ?? 'none',
  }
}

export const liarsBarGameModule: GameModule = {
  gameId: 'liars-bar',
  Workspace: LiarsBarWorkspace,
  getSummary: getLiarsBarSummary,
}
