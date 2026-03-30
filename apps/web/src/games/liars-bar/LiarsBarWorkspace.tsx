import type { GameSession } from '@human-agent-playground/core'

import { getSideLabel, useI18n } from '../../i18n'
import { ActivityFeed } from '../shared/ActivityFeed'
import type { GameWorkspaceProps } from '../types'

type LiarsBarCardRank = 'king' | 'queen' | 'ace' | 'joker'

interface LiarsBarPreviousPlay {
  side: string
  displayName: string
  declaredRank: LiarsBarCardRank
  cardCount: number
}

interface LiarsBarPublicPlayer {
  side: string
  displayName: string
  alive: boolean
  shotsFired: number
  handSize: number
}

interface LiarsBarPublicState {
  kind: 'liars-bar'
  turn: string
  status: 'active' | 'finished'
  winner: string | null
  roundNumber: number
  phase: string
  tableCard: LiarsBarCardRank
  previousPlay: LiarsBarPreviousPlay | null
  players: LiarsBarPublicPlayer[]
}

function toLiarsBarSession(session: GameSession): GameSession<LiarsBarPublicState> {
  if (
    session.gameId !== 'liars-bar' ||
    typeof session.state !== 'object' ||
    session.state === null ||
    (session.state as { kind?: unknown }).kind !== 'liars-bar'
  ) {
    throw new Error('Invalid Liars Bar session payload')
  }

  return session as GameSession<LiarsBarPublicState>
}

function formatCardName(rank: LiarsBarCardRank) {
  switch (rank) {
    case 'king':
      return 'King'
    case 'queen':
      return 'Queen'
    case 'ace':
      return 'Ace'
    case 'joker':
      return 'Joker'
  }
}

export function LiarsBarWorkspace({
  session: rawSession,
  sideRailHeader,
  setupPanel,
}: GameWorkspaceProps) {
  const { language, t } = useI18n()
  const session = toLiarsBarSession(rawSession)
  const sessionEvents = [...session.events].sort((left, right) =>
    left.createdAt.localeCompare(right.createdAt),
  )

  const previousPlay = session.state.previousPlay
  const faceDownCards = Array.from({ length: previousPlay?.cardCount ?? 0 }, (_, index) => index)

  return (
    <div className="game-workspace-layout game-workspace-layout-text">
      <article className="board-panel text-workspace-panel">
        <section className="panel-card liars-bar-status-strip" aria-label={t('liarsBar.statusStrip')}>
          <span>{t('liarsBar.round', { value: session.state.roundNumber })}</span>
          <span>{t('liarsBar.turn', { value: getSideLabel(language, session.state.turn) })}</span>
          <span>{t('liarsBar.phase', { value: session.state.phase })}</span>
        </section>

        <ActivityFeed
          gameId={session.gameId}
          events={sessionEvents}
          emptyText={t('feed.empty')}
          pendingItem={setupPanel}
          layout="timeline"
        />
      </article>

      <aside className="side-panel">
        <section className="panel-card liars-bar-table-side-card" aria-label={t('liarsBar.tableArea')}>
          <div className="liars-bar-table-side-grid">
            <div className="liars-bar-table-side-column">
              <p className="eyebrow">{t('liarsBar.tableCard')}</p>
              <div className="liars-bar-card liars-bar-card-face" data-card-rank={session.state.tableCard}>
                <span className="liars-bar-card-corner">{formatCardName(session.state.tableCard)}</span>
                <strong>{formatCardName(session.state.tableCard)}</strong>
                <span className="liars-bar-card-subtitle">{t('liarsBar.tableCard')}</span>
              </div>
            </div>

            <div className="liars-bar-table-side-column">
              <p className="eyebrow">{t('liarsBar.faceDownPile')}</p>
              {faceDownCards.length > 0 ? (
                <div
                  className="liars-bar-face-down-pile"
                  aria-label={t('liarsBar.faceDownCards', { value: faceDownCards.length })}
                >
                  {faceDownCards.map((offset) => (
                    <div
                      key={offset}
                      className="liars-bar-card liars-bar-card-back"
                      style={{ transform: `translate(${offset * 8}px, ${offset * -2}px)` }}
                    />
                  ))}
                </div>
              ) : (
                <p className="liars-bar-table-empty">{t('liarsBar.noPreviousPlay')}</p>
              )}
            </div>
          </div>
          <p className="liars-bar-table-side-meta">
            {faceDownCards.length > 0
              ? t('liarsBar.currentPileSummary', {
                  player: previousPlay?.displayName ?? getSideLabel(language, session.state.turn),
                  count: faceDownCards.length,
                })
              : t('liarsBar.noPreviousPlay')}
          </p>
        </section>

        <section className="panel-card liars-bar-public-players">
          <div className="players-panel-header">
            <h2>{t('players.title')}</h2>
            {sideRailHeader}
          </div>

          <div className="liars-bar-public-player-list">
            {session.state.players.map((player) => {
              const isCurrentTurn = player.side === session.state.turn && session.state.status !== 'finished'
              return (
                <article
                  key={player.side}
                  className={`liars-bar-public-player ${isCurrentTurn ? 'is-current-turn' : ''} ${player.alive ? '' : 'is-eliminated'}`.trim()}
                >
                  <div className="liars-bar-public-player-header">
                    <strong>{player.displayName}</strong>
                    <div className="liars-bar-public-player-meta">
                      <span>{player.alive ? t('liarsBar.playerActive') : t('liarsBar.playerEliminated')}</span>
                      <span>{t('liarsBar.revolverCompact', { value: `${player.shotsFired}/6` })}</span>
                      <span>{t('liarsBar.handSizeCompact', { value: player.handSize })}</span>
                    </div>
                  </div>
                </article>
              )
            })}
          </div>
        </section>
      </aside>
    </div>
  )
}
