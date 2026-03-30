import type { GameSession } from '@human-agent-playground/core'
import { render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { PendingThinkingFeedItem } from '../../shared/ActivityFeed'
import { LiarsBarWorkspace } from '../LiarsBarWorkspace'

function createSession(): GameSession {
  return {
    id: 'session-liars-1',
    gameId: 'liars-bar',
    createdAt: '2026-03-29T01:00:00.000Z',
    updatedAt: '2026-03-29T01:00:10.000Z',
    state: {
      kind: 'liars-bar',
      turn: 'seat-2',
      status: 'active',
      winner: null,
      roundNumber: 2,
      phase: 'forced_challenge',
      tableCard: 'queen',
      previousPlay: {
        side: 'seat-1',
        displayName: 'P1',
        declaredRank: 'queen',
        cardCount: 2,
      },
      lastResolution: {
        callerSide: 'seat-4',
        callerDisplayName: 'P4',
        accusedSide: 'seat-3',
        accusedDisplayName: 'P3',
        punishedSide: 'seat-3',
        punishedDisplayName: 'P3',
        revealedCards: ['queen', 'ace'],
        liarFound: true,
        shotsFired: 2,
        survived: true,
        nextRoundNumber: 2,
        nextTurn: 'seat-3',
        winner: null,
      },
      players: [
        {
          side: 'seat-1',
          displayName: 'P1',
          alive: true,
          shotsFired: 0,
          handSize: 3,
        },
        {
          side: 'seat-2',
          displayName: 'P2',
          alive: true,
          shotsFired: 1,
          handSize: 1,
        },
        {
          side: 'seat-3',
          displayName: 'P3',
          alive: true,
          shotsFired: 2,
          handSize: 4,
        },
        {
          side: 'seat-4',
          displayName: 'P4',
          alive: false,
          shotsFired: 3,
          handSize: 0,
        },
      ],
    },
    events: [
      {
        id: 'event-created',
        kind: 'session_created',
        createdAt: '2026-03-29T01:00:00.000Z',
        actorKind: 'system',
        channel: 'system',
        summary: 'Created a new Liars Bar session.',
        details: { gameId: 'liars-bar' },
      },
      {
        id: 'event-play-1',
        kind: 'action_taken',
        createdAt: '2026-03-29T01:00:05.000Z',
        actorKind: 'agent',
        channel: 'system',
        actorName: 'restflow-bridge',
        summary: 'P1 played 2 cards face down as Queens.',
        details: {
          headline: 'P1 plays 2 cards as Queens',
          actionType: 'play_cards',
          side: 'seat-1',
          cardCount: 2,
          tableCard: 'queen',
        },
      },
    ],
    aiSeats: {
      'seat-1': {
        side: 'seat-1',
        launcher: 'openai',
        enabled: true,
        autoPlay: true,
        model: 'gpt-5',
        providerProfileId: 'profile-1',
        promptOverride: null,
        timeoutMs: 300000,
        status: 'idle',
        lastError: null,
        runtimeSource: 'restflow-bridge',
      },
      'seat-2': {
        side: 'seat-2',
        launcher: 'claude_code',
        enabled: true,
        autoPlay: true,
        model: 'claude-code-sonnet',
        providerProfileId: null,
        promptOverride: null,
        timeoutMs: 300000,
        status: 'thinking',
        lastError: null,
        runtimeSource: 'restflow-bridge',
      },
    },
  }
}

describe('LiarsBarWorkspace', () => {
  it('renders a text-first liar deck workspace with public revolver state and a pending thinking item', () => {
    render(
      <LiarsBarWorkspace
        game={{
          id: 'liars-bar',
          title: 'Liars Bar',
          shortName: 'Liars Bar',
          description: 'Text-first bluffing game.',
          sides: ['seat-1', 'seat-2', 'seat-3', 'seat-4'],
        }}
        session={createSession()}
        error={null}
        onSessionUpdate={vi.fn()}
        onError={vi.fn()}
        sideRailHeader={<div>Seat Runtime</div>}
        setupPanel={
          <PendingThinkingFeedItem
            launcherLabel="Claude Code"
            sideLabel="P2"
            startedAt="2026-03-29T01:00:08.000Z"
          />
        }
      />,
    )

    expect(screen.getByText('Round 2')).toBeInTheDocument()
    expect(screen.getByLabelText("Liar's Deck round status")).toBeInTheDocument()
    expect(screen.getByText('P1 · 2 face-down cards')).toBeInTheDocument()
    expect(screen.getByLabelText("Liar's Deck table")).toBeInTheDocument()
    expect(screen.getByLabelText('2 face-down cards')).toBeInTheDocument()
    expect(screen.getByText('Seat Runtime')).toBeInTheDocument()
    expect(screen.getByText('P4')).toBeInTheDocument()
    expect(screen.getByText('eliminated')).toBeInTheDocument()
    expect(screen.getByText('R 2/6')).toBeInTheDocument()
    expect(screen.getByText('Hand 4')).toBeInTheDocument()
    expect(screen.getByText(/P2 · Claude Code is thinking/i)).toBeInTheDocument()
    expect(screen.getAllByRole('heading', { name: 'Players' })).toHaveLength(1)
    expect(screen.queryByText('LIAR')).toBeNull()

    const messageFeedCard = screen
      .getByRole('heading', { name: 'Message Feed' })
      .closest('.panel-card')
    expect(messageFeedCard).not.toBeNull()
    expect(messageFeedCard).toHaveClass('panel-card-feed-timeline')
    expect(
      within(messageFeedCard as HTMLElement).getByText('P1 plays 2 cards as Queens'),
    ).toBeInTheDocument()
    expect(document.querySelector('[data-square]')).toBeNull()
  })
})
