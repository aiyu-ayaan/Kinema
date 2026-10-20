import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router-dom'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import {
  ArrowLeft,
  ArrowsIn,
  ArrowsOut,
  CircleNotch,
  ClockCounterClockwise,
  ClockClockwise,
  Pause,
  Play,
  SkipForward,
  SpeakerHigh,
  SpeakerSlash,
  Stop,
  Subtitles,
  Check,
} from '@phosphor-icons/react'
import { api, type PlayCaps } from '@/lib/api'
import { cn } from '@/lib/cn'
import { hasTopLeftWindowControls, hasTopRightWindowControls } from '@/lib/desktop'
import { clock, isTextSub, subtitleOf, trackLabel } from '@/lib/format'
import { t } from '@/lib/i18n'
import type { ItemDetail, PlaybackPlan, Stream } from '@/lib/types'
import { parseVtt, type Cue } from '@/lib/vtt'


/** Loudest volume: 200 %, the upper 100 % being boost (as in MX Player). */
const MAX_LEVEL = 2

/** What the browser decodes beyond the baseline; the server picks direct play, remux or transcode from it. */
function detectCaps(): PlayCaps {
  const ms = typeof MediaSource !== 'undefined' ? MediaSource : null
  const v = document.createElement('video')
  const ok = (type: string) => !!ms?.isTypeSupported(type) || v.canPlayType(type) === 'probably'
  return {
    hevc: ok('video/mp4; codecs="hvc1.1.6.L93.B0"'),
    av1: ok('video/mp4; codecs="av01.0.05M.08"'),
  }
}

function IconButton({ label, onClick, children, className }: { label: string; onClick: () => void; children: ReactNode; className?: string }) {
  return (
    <button type="button" onClick={onClick} aria-label={label} title={label} className={cn('flex h-10 w-10 cursor-pointer items-center justify-center rounded-full text-white/90 transition-colors hover:bg-white/15 hover:text-white', className)}>
      {children}
    </button>
  )
}

function TrackMenu({
  audios,
  subs,
  audio,
  sub,
  onAudio,
  onSub,
}: {
  audios: Stream[]
  subs: Stream[]
  audio: number | null
  sub: number | null
  onAudio: (idx: number) => void
  onSub: (id: number | null) => void
}) {
  const option = (label: string, active: boolean, onClick: () => void, disabled = false, note?: string) => (
    <li key={label}>
      <button
        type="button"
        disabled={disabled}
        onClick={onClick}
        className="flex w-full cursor-pointer items-center gap-2 rounded-md px-3 py-2 text-start text-label text-white/85 hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-40"
      >
        <Check size={14} weight="bold" className={active ? 'text-primary' : 'invisible'} aria-hidden />
        <span className="min-w-0 flex-1">{label}</span>
        {note && <span className="text-caption text-white/50">{note}</span>}
      </button>
    </li>
  )
  return (
    <div className="anim-enter absolute bottom-28 end-4 z-10 grid w-[min(560px,calc(100vw-2rem))] grid-cols-1 gap-4 rounded-lg border border-white/10 bg-black/85 p-3 shadow-overlay backdrop-blur-md sm:grid-cols-2">
      <div>
        <p className="px-3 pb-1 text-caption font-semibold uppercase tracking-wider text-white/50">{t('player.audio')}</p>
        <ul className="max-h-64 overflow-y-auto">{audios.map((a, i) => option(trackLabel(a, i + 1), a.idx === audio, () => onAudio(a.idx)))}</ul>
      </div>
      <div>
        <p className="px-3 pb-1 text-caption font-semibold uppercase tracking-wider text-white/50">{t('player.subtitles')}</p>
        <ul className="max-h-64 overflow-y-auto">
          {option(t('player.subtitlesOff'), sub == null, () => onSub(null))}
          {subs.map((s, i) => {
            const text = isTextSub(s)
            return option(trackLabel(s, i + 1), s.id === sub, () => onSub(s.id), !text, text ? undefined : t('player.subtitleImage'))
          })}
        </ul>
      </div>
    </div>
  )
}

function Player({ detail, startAt }: { detail: ItemDetail; startAt: number }) {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const { item } = detail
  const file = detail.files[0]
  const caps = useMemo(detectCaps, [])
  const audios = useMemo(() => file.streams.filter((s) => s.type === 'audio'), [file])
  const subs = useMemo(() => file.streams.filter((s) => s.type === 'subtitle'), [file])

  const videoRef = useRef<HTMLVideoElement>(null)
  const shellRef = useRef<HTMLDivElement>(null)
  const [plan, setPlan] = useState<PlaybackPlan | null>(null)
  const [src, setSrc] = useState<string | null>(null)
  // null until the first plan: the server picks the track from the language settings.
  const [audio, setAudio] = useState<number | null>(null)
  const [subId, setSubId] = useState<number | null>(null)
  const [cues, setCues] = useState<Cue[]>([])
  const [time, setTime] = useState(startAt)
  const [scrub, setScrub] = useState<number | null>(null)
  const [playing, setPlaying] = useState(false)
  const [waiting, setWaiting] = useState(true)
  const [muted, setMuted] = useState(false)
  const [volume, setVolume] = useState(1)
  // Gain above 100 % (MX Player style boost) through Web Audio; 1 means the element's own volume only.
  const [boost, setBoost] = useState(1)
  const audioGraph = useRef<{ ctx: AudioContext; gain: GainNode } | null>(null)
  const [fullscreen, setFullscreen] = useState(false)
  const [menu, setMenu] = useState(false)
  const [chrome, setChrome] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Remux/transcode streams start at `offset`, so absolute time = offset + currentTime.
  const offset = useRef(0)
  const pendingSeek = useRef<number | null>(null)
  const mode = useRef<PlaybackPlan['mode']>('direct')
  const watchedSent = useRef(false)
  const planned = useRef(false)
  /** Absolute time being sought to; timeupdates are ignored until it lands. */
  const seekTarget = useRef<number | null>(null)
  const loadSeq = useRef(0)

  const absTime = useCallback(() => {
    const v = videoRef.current
    return v ? offset.current + v.currentTime : 0
  }, [])

  const duration = plan?.duration_s ?? (videoRef.current && Number.isFinite(videoRef.current.duration) ? videoRef.current.duration : null)

  const load = useCallback(
    async (audioIdx: number | null, at: number) => {
      setError(null)
      setWaiting(true)
      // Show the target right away; the plan request can take a moment on big files.
      setTime(at)
      seekTarget.current = at
      const seq = ++loadSeq.current
      try {
        const p = await api.plan(file.id, audioIdx, caps, at)
        // A newer seek or track change started meanwhile: its answer wins.
        if (seq !== loadSeq.current) return
        // Language settings pick the starting subtitle; later choices are the viewer's.
        if (!planned.current && p.subtitle != null) setSubId(p.subtitle)
        planned.current = true
        setAudio(p.audio)
        setPlan(p)
        mode.current = p.mode
        if (p.mode === 'direct') {
          offset.current = 0
          pendingSeek.current = at
          setSrc(api.streamUrl(file.id, audioIdx, caps, 0))
        } else {
          // ffmpeg gets the requested time; p.start is where its output really begins.
          offset.current = p.start
          pendingSeek.current = null
          setSrc(api.streamUrl(file.id, audioIdx, caps, at))
        }
      } catch (e) {
        if (seq !== loadSeq.current) return
        seekTarget.current = null
        setError(e instanceof Error ? e.message : String(e))
      }
    },
    [file.id, caps],
  )

  useEffect(() => {
    load(audio, startAt)
    // Only on mount; track changes call load themselves.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const seek = useCallback(
    (to: number) => {
      const v = videoRef.current
      if (!v) return
      const target = Math.max(0, duration ? Math.min(to, duration - 1) : to)
      if (mode.current === 'direct') {
        seekTarget.current = target
        setTime(target)
        v.currentTime = target
        return
      }
      // Inside what this stream already holds: seek locally instead of restarting ffmpeg.
      const local = target - offset.current
      for (let i = 0; i < v.buffered.length; i++) {
        if (local >= v.buffered.start(i) && local <= v.buffered.end(i)) {
          seekTarget.current = target
          setTime(target)
          v.currentTime = local
          return
        }
      }
      load(audio, target)
    },
    [audio, duration, load],
  )

  const togglePlay = useCallback(() => {
    const v = videoRef.current
    if (!v) return
    if (v.paused) v.play().catch(() => {})
    else v.pause()
  }, [])

  const level = muted ? 0 : volume * boost

  /** 0..MAX_LEVEL: up to 1 is the element's volume, above it a gain node boosts quiet files. */
  const setLevel = useCallback((to: number) => {
    const v = videoRef.current
    if (!v) return
    const l = Math.min(MAX_LEVEL, Math.max(0, Math.round(to * 100) / 100))
    v.volume = Math.min(1, l)
    v.muted = l === 0
    const gain = Math.max(1, l)
    // Built on first boost only: once an element feeds Web Audio it can never go back, and the context needs a user gesture.
    if (gain > 1 && !audioGraph.current) {
      try {
        const ctx = new AudioContext()
        const node = ctx.createGain()
        // A limiter so the boost gets louder instead of clipping into distortion.
        const limiter = ctx.createDynamicsCompressor()
        limiter.threshold.value = -1
        limiter.ratio.value = 20
        ctx.createMediaElementSource(v).connect(node).connect(limiter).connect(ctx.destination)
        audioGraph.current = { ctx, gain: node }
      } catch {
        setBoost(1)
        return
      }
    }
    const graph = audioGraph.current
    if (graph) {
      graph.gain.gain.value = gain
      graph.ctx.resume().catch(() => {})
    }
    setBoost(gain)
  }, [])
  useEffect(() => () => void audioGraph.current?.ctx.close().catch(() => {}), [])

  const toggleFullscreen = useCallback(() => {
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {})
    else shellRef.current?.requestFullscreen().catch(() => {})
  }, [])

  const report = useCallback(
    (watched = false) => {
      const pos = absTime()
      if (pos > 0) api.progress(item.id, pos, watched).catch(() => {})
    },
    [absTime, item.id],
  )

  const stop = useCallback(() => {
    const v = videoRef.current
    if (v) v.pause()
    if (!watchedSent.current) report()
    if (document.fullscreenElement) {
      document.exitFullscreen().catch(() => {})
    }
    if (window.history.length > 1) {
      navigate(-1)
    } else {
      navigate(`/item/${item.id}`, { replace: true })
    }
  }, [item.id, navigate, report])

  // Progress every 10 s while playing, and once on leaving.
  useEffect(() => {
    if (!playing) return
    const id = window.setInterval(() => report(), 10_000)
    return () => window.clearInterval(id)
  }, [playing, report])
  useEffect(
    () => () => {
      if (!watchedSent.current) report()
      qc.invalidateQueries({ queryKey: ['home'] })
      qc.invalidateQueries({ queryKey: ['item'] })
    },
    [report, qc],
  )

  // Subtitles are fetched once per track and drawn over the video.
  useEffect(() => {
    if (subId == null) {
      setCues([])
      return
    }
    let cancelled = false
    api
      .subtitleText(subId)
      .then((text) => !cancelled && setCues(parseVtt(text)))
      .catch(() => !cancelled && setCues([]))
    return () => {
      cancelled = true
    }
  }, [subId])
  const activeCues = cues.filter((c) => c.start <= time && time < c.end)

  // Controls fade after 3 s without pointer movement while playing.
  const hideTimer = useRef<number>()
  const poke = useCallback(() => {
    setChrome(true)
    window.clearTimeout(hideTimer.current)
    hideTimer.current = window.setTimeout(() => setChrome(false), 3000)
  }, [])
  useEffect(() => () => window.clearTimeout(hideTimer.current), [])
  const showChrome = chrome || !playing || menu

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement && e.target.type !== 'range') return
      const v = videoRef.current
      if (!v) return
      if (e.key === ' ' || e.key === 'k') {
        e.preventDefault()
        togglePlay()
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault()
        seek(absTime() - 10)
      } else if (e.key === 'ArrowRight') {
        e.preventDefault()
        seek(absTime() + 10)
      } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
        e.preventDefault()
        setLevel(level + (e.key === 'ArrowUp' ? 0.05 : -0.05))
      } else if (e.key === 'f') toggleFullscreen()
      else if (e.key === 'm') v.muted = !v.muted
      else if (e.key === 's' || e.key === 'x') {
        e.preventDefault()
        stop()
      } else if (e.key === 'Escape' && !document.fullscreenElement) stop()
      poke()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [absTime, level, poke, seek, setLevel, stop, toggleFullscreen, togglePlay])

  useEffect(() => {
    const onChange = () => setFullscreen(!!document.fullscreenElement)
    document.addEventListener('fullscreenchange', onChange)
    return () => document.removeEventListener('fullscreenchange', onChange)
  }, [])

  // The pick is saved for this profile, so the Android app and other browsers start this title the same way.
  const saveTracks = (audioIdx: number | null, sub: number | null) => {
    const a = audios.findIndex((s) => s.idx === audioIdx)
    const pos = subs.findIndex((st) => st.id === sub)
    const s = sub == null ? -1 : pos >= 0 ? pos : null
    api.setTrackChoice(item.id, a >= 0 ? a : null, s).catch(() => {})
  }

  const shown = scrub ?? time
  const fill = duration ? `${(shown / duration) * 100}%` : '0%'
  const title = item.kind === 'episode' ? (item.show_title ?? item.title) : item.title
  const subtitle = item.kind === 'episode' ? `${subtitleOf(item)} · ${item.title}` : item.year ? String(item.year) : null

  return (
    <div
      ref={shellRef}
      onMouseMove={poke}
      onClick={() => menu && setMenu(false)}
      className={cn('fixed inset-0 z-sheet bg-black text-white', !showChrome && 'cursor-none')}
    >
      <video
        ref={videoRef}
        src={src ?? undefined}
        autoPlay
        playsInline
        className="h-full w-full object-contain"
        onClick={togglePlay}
        onDoubleClick={toggleFullscreen}
        onLoadedMetadata={(e) => {
          // A new stream: remux/transcode seeks are done once it has loaded.
          if (pendingSeek.current) e.currentTarget.currentTime = pendingSeek.current
          else seekTarget.current = null
          pendingSeek.current = null
          e.currentTarget.play().catch(() => {})
        }}
        onSeeked={() => {
          seekTarget.current = null
        }}
        onTimeUpdate={() => {
          // Until a seek lands, the old stream's clock must not pull the slider back.
          if (seekTarget.current != null) return
          const now = absTime()
          setTime(now)
          if (!watchedSent.current && duration && now / duration > 0.92) {
            watchedSent.current = true
            report(true)
          }
        }}
        onPlay={() => setPlaying(true)}
        onPause={() => setPlaying(false)}
        onWaiting={() => setWaiting(true)}
        onPlaying={() => setWaiting(false)}
        onCanPlay={() => setWaiting(false)}
        onVolumeChange={(e) => {
          setMuted(e.currentTarget.muted)
          setVolume(e.currentTarget.volume)
        }}
        onEnded={() => {
          if (!watchedSent.current) {
            watchedSent.current = true
            report(true)
          }
          if (detail.next_id) navigate(`/play/${detail.next_id}?t=0`, { replace: true })
        }}
        onError={() => {
          seekTarget.current = null
          setWaiting(false)
          setError(t('player.error'))
        }}
      />

      {activeCues.length > 0 && (
        <div className={cn('pointer-events-none absolute inset-x-0 flex flex-col items-center gap-1 px-6 text-center transition-all duration-slow', showChrome ? 'bottom-28' : 'bottom-12')}>
          {activeCues.map((c, i) => (
            <p key={i} className="text-over-media max-w-4xl whitespace-pre-line rounded bg-black/40 px-2 py-0.5 text-[clamp(16px,2.4vw,32px)] font-semibold leading-snug">
              {c.text}
            </p>
          ))}
        </div>
      )}

      {waiting && !error && (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <CircleNotch size={48} className="animate-spin text-primary" aria-label={t('common.loading')} />
        </div>
      )}

      {error && (
        <div className="absolute inset-0 flex items-center justify-center p-6">
          <div className="max-w-md rounded-lg border border-white/10 bg-black/80 p-6 text-center">
            <p className="text-title font-semibold">{t('player.errorTitle')}</p>
            <p className="mt-2 text-body text-white/70">{error}</p>
          </div>
        </div>
      )}

      {/* Top bar */}
      <div
        className={cn(
          'absolute inset-x-0 top-0 flex items-center gap-3 bg-gradient-to-b from-black/80 to-transparent p-4 transition-opacity duration-slow',
          showChrome ? 'opacity-100' : 'pointer-events-none opacity-0',
          hasTopRightWindowControls && 'pe-40 md:pe-44',
          hasTopLeftWindowControls && 'ps-20',
        )}
      >
        <IconButton label={t('common.back')} onClick={stop}>
          <ArrowLeft size={22} />
        </IconButton>
        <div className="min-w-0">
          <p className="truncate text-title font-semibold">{title}</p>
          {subtitle && <p className="truncate text-label text-white/70">{subtitle}</p>}
        </div>
        {plan && (
          <span className="ms-auto rounded border border-white/20 px-2 py-0.5 text-caption font-medium text-white/70" title={t(`player.mode.${plan.mode}.hint`)}>
            {t(`player.mode.${plan.mode}`)}
          </span>
        )}
      </div>

      {menu && (
        <div onClick={(e) => e.stopPropagation()}>
          <TrackMenu
            audios={audios}
            subs={subs}
            audio={audio}
            sub={subId}
            onAudio={(idx) => {
              setMenu(false)
              if (idx === audio) return
              setAudio(idx)
              saveTracks(idx, subId)
              load(idx, absTime())
            }}
            onSub={(id) => {
              setSubId(id)
              setMenu(false)
              saveTracks(audio, id)
            }}
          />
        </div>
      )}

      {/* Bottom controls */}
      <div
        onClick={(e) => e.stopPropagation()}
        className={cn('absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent px-4 pb-3 pt-12 transition-opacity duration-slow', showChrome ? 'opacity-100' : 'pointer-events-none opacity-0')}
      >
        {duration ? (
          <div className="flex items-center gap-3 px-2">
            <span className="w-16 text-end text-label tabular text-white/80">{clock(shown)}</span>
            <input
              type="range"
              className="range flex-1"
              min={0}
              max={duration}
              step={1}
              value={shown}
              style={{ ['--fill' as string]: fill }}
              aria-label={t('player.seek')}
              onChange={(e) => setScrub(Number(e.target.value))}
              onPointerUp={(e) => {
                const target = Number(e.currentTarget.value)
                seek(target)
                setScrub(null)
              }}
              onKeyUp={(e) => {
                const target = Number(e.currentTarget.value)
                seek(target)
                setScrub(null)
              }}
            />
            <span className="w-16 text-label tabular text-white/60">{clock(duration)}</span>
          </div>
        ) : null}
        <div className="mt-1 flex items-center gap-1">
          <IconButton label={t('player.back10')} onClick={() => seek(absTime() - 10)}>
            <ClockCounterClockwise size={22} />
          </IconButton>
          <IconButton label={playing ? t('player.pause') : t('player.play')} onClick={togglePlay} className="h-12 w-12">
            {playing ? <Pause size={28} weight="fill" /> : <Play size={28} weight="fill" />}
          </IconButton>
          <IconButton label={t('player.forward10')} onClick={() => seek(absTime() + 10)}>
            <ClockClockwise size={22} />
          </IconButton>
          <IconButton label={t('player.stop')} onClick={stop}>
            <Stop size={22} weight="fill" />
          </IconButton>
          {detail.next_id && (
            <IconButton label={t('player.next')} onClick={() => navigate(`/play/${detail.next_id}?t=0`, { replace: true })}>
              <SkipForward size={22} weight="fill" />
            </IconButton>
          )}
          <div className="group ms-2 flex items-center">
            <IconButton
              label={muted ? t('player.unmute') : t('player.mute')}
              onClick={() => {
                const v = videoRef.current
                if (v) v.muted = !v.muted
              }}
            >
              {level === 0 ? <SpeakerSlash size={22} /> : <SpeakerHigh size={22} />}
            </IconButton>
            <input
              type="range"
              className="range w-0 opacity-0 transition-all duration-base group-hover:w-32 group-hover:opacity-100 focus:w-32 focus:opacity-100"
              min={0}
              max={MAX_LEVEL}
              step={0.05}
              value={level}
              style={{ ['--fill' as string]: `${(level / MAX_LEVEL) * 100}%` }}
              aria-label={t('player.volume')}
              aria-valuetext={`${Math.round(level * 100)}%`}
              onChange={(e) => setLevel(Number(e.target.value))}
            />
            {level > 1 && (
              <span className="ms-2 rounded bg-primary px-1.5 py-0.5 text-caption font-semibold tabular text-on-primary" title={t('player.boost')}>
                {Math.round(level * 100)}%
              </span>
            )}
          </div>
          <div className="ms-auto flex items-center gap-1">
            {(audios.length > 1 || subs.length > 0) && (
              <IconButton label={t('player.tracks')} onClick={() => setMenu((m) => !m)} className={menu ? 'bg-white/15' : undefined}>
                <Subtitles size={22} weight={subId != null ? 'fill' : 'regular'} />
              </IconButton>
            )}
            <IconButton label={fullscreen ? t('player.exitFullscreen') : t('player.fullscreen')} onClick={toggleFullscreen}>
              {fullscreen ? <ArrowsIn size={22} /> : <ArrowsOut size={22} />}
            </IconButton>
          </div>
        </div>
      </div>
    </div>
  )
}

export function PlayerPage() {
  const id = Number(useParams().id)
  const [params] = useSearchParams()
  const { data, error } = useQuery({ queryKey: ['item', id], queryFn: () => api.item(id), staleTime: 0 })
  if (error || (data && data.files.length === 0)) {
    return (
      <div className="fixed inset-0 z-sheet flex items-center justify-center bg-black p-6 text-center text-white/80">
        {error instanceof Error ? error.message : t('player.noFile')}
      </div>
    )
  }
  if (!data) {
    return (
      <div className="fixed inset-0 z-sheet flex items-center justify-center bg-black">
        <CircleNotch size={48} className="animate-spin text-primary" aria-label={t('common.loading')} />
      </div>
    )
  }
  const explicit = params.get('t')
  const resume = data.item.watched ? 0 : (data.item.position_s ?? 0)
  return <Player key={id} detail={data} startAt={explicit != null ? Number(explicit) || 0 : resume} />
}

export default PlayerPage
