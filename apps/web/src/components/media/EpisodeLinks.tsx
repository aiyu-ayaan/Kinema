import { Link } from 'react-router-dom'
import { t } from '@/lib/i18n'
import type { ItemCard } from '@/lib/types'

/** "S2 · E1" where S2 opens the season and E1 the episode's page. */
export function EpisodeLinks({ item, className = 'hover:text-fg hover:underline' }: { item: ItemCard; className?: string }) {
  const season = t('item.seasonNo', { s: item.season_no ?? 0 })
  return (
    <>
      {item.parent_id != null ? (
        <Link to={`/item/${item.parent_id}`} className={className}>
          {season}
        </Link>
      ) : (
        season
      )}
      {' · '}
      <Link to={`/item/${item.id}`} className={className}>
        {t('item.episodeNo', { e: item.episode_no ?? 0 })}
      </Link>
    </>
  )
}
