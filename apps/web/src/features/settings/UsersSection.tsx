import { useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowsLeftRight, Check, Crown, HouseLine, LockSimple, PencilSimple, Plus, ShieldCheck, ShieldStar, Trash } from '@phosphor-icons/react'
import { Avatar } from '@/components/ui/Avatar'
import { avatarColors } from '@/components/ui/avatarColors'
import { Button } from '@/components/ui/Button'
import { Badge, Card, CardHeader, ErrorText, Skeleton } from '@/components/ui/Card'
import { Select, SettingRow, Switch } from '@/components/ui/Controls'
import { Field } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { useAuth } from '@/lib/auth'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'
import { useLibraries } from '@/lib/queries'
import type { AvatarColor, ManagedUser, Prefs, ProfileInput } from '@/lib/types'
import { LanguageFields, TransferModal } from '@/features/profiles/ProfileFields'
import { ratingLabel } from '@/features/profiles/rating'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))
const NO_PREFS: Prefs = { audio_language: null, subtitle_language: null, subtitle_mode: null, max_rating: null, allow_unrated: false, lan_unlock: false }
const colors = Object.keys(avatarColors) as AvatarColor[]

/** Add or edit a profile. `user` null means a new one. */
function ProfileEditor({ user, open, onClose }: { user: ManagedUser | null; open: boolean; onClose: () => void }) {
  const qc = useQueryClient()
  const { user: me } = useAuth()
  const { data: libraries } = useLibraries()
  const [name, setName] = useState(user?.name ?? '')
  const [avatar, setAvatar] = useState<AvatarColor>(user?.avatar ?? colors[Math.floor(Math.random() * colors.length)])
  const [isAdmin, setIsAdmin] = useState(user?.is_admin ?? false)
  const [password, setPassword] = useState('')
  const [pin, setPin] = useState('')
  const [removePin, setRemovePin] = useState(false)
  const [limited, setLimited] = useState((user?.library_ids.length ?? 0) > 0)
  const [libraryIds, setLibraryIds] = useState<number[]>(user?.library_ids ?? [])
  const [prefs, setPrefs] = useState<Prefs>(user?.preferences ?? NO_PREFS)
  const { data: mine } = useQuery({ queryKey: ['my-profile'], queryFn: api.myProfile })
  const willHavePin = !removePin && (pin !== '' || !!user?.has_pin)

  const save = useMutation({
    mutationFn: (p: ProfileInput) => (user ? api.updateUser(user.id, p) : api.createUser(p)),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['users'] })
      qc.invalidateQueries({ queryKey: ['profiles'] })
      onClose()
    },
  })

  const needsPassword = isAdmin && !user?.has_password
  const valid = name.trim().length > 0 && (!needsPassword || password.length >= 8) && (pin === '' || /^\d{4,8}$/.test(pin)) && (!limited || isAdmin || libraryIds.length > 0)

  function submit(e: FormEvent) {
    e.preventDefault()
    if (!valid) return
    save.mutate({
      name: name.trim(),
      avatar,
      is_admin: isAdmin,
      // Blank fields keep what the profile already has.
      password: password || undefined,
      pin: removePin ? '' : pin || undefined,
      library_ids: !isAdmin && limited ? libraryIds : [],
      // Admins are never limited and always need their password.
      preferences: isAdmin ? { ...prefs, max_rating: null, allow_unrated: false, lan_unlock: false } : { ...prefs, lan_unlock: prefs.lan_unlock && willHavePin },
    })
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={user ? t('users.edit') : t('users.add')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button variant="primary" type="submit" form="profile-form" loading={save.isPending} disabled={!valid}>
            {t('common.save')}
          </Button>
        </>
      }
    >
      <form id="profile-form" onSubmit={submit} className="flex flex-col gap-5">
        <div className="flex items-center gap-4">
          <Avatar name={name || '?'} color={avatar} size={64} />
          <Field className="flex-1" label={t('users.name')} required maxLength={40} value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-label font-medium">{t('users.color')}</span>
          <div className="flex flex-wrap gap-2" role="radiogroup" aria-label={t('users.color')}>
            {colors.map((c) => (
              <button
                key={c}
                type="button"
                role="radio"
                aria-checked={avatar === c}
                aria-label={c}
                onClick={() => setAvatar(c)}
                className={cn('flex h-9 w-9 cursor-pointer items-center justify-center rounded-full ring-offset-2 ring-offset-panel transition-shadow', avatar === c && 'ring-2 ring-fg')}
                style={{ background: avatarColors[c] }}
              >
                {avatar === c && <Check size={16} weight="bold" className="text-white" />}
              </button>
            ))}
          </div>
        </div>

        <div className="-mx-4 divide-y divide-border border-y border-border">
          <SettingRow label={t('users.admin')} description={t('users.adminHint')}>
            <Switch checked={isAdmin} onChange={setIsAdmin} disabled={user?.is_owner} label={t('users.admin')} />
          </SettingRow>
        </div>

        {isAdmin ? (
          <Field
            label={t('auth.password')}
            type="password"
            autoComplete="new-password"
            minLength={8}
            required={needsPassword}
            hint={user?.has_password ? t('users.passwordKeep') : t('auth.atLeast8')}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        ) : (
          <div className="flex flex-col gap-2">
            <Field
              label={t('users.pin')}
              inputMode="numeric"
              autoComplete="off"
              maxLength={8}
              disabled={removePin}
              placeholder={user?.has_pin ? '••••' : ''}
              hint={user?.has_pin ? t('users.pinKeep') : t('users.pinHint')}
              error={pin && !/^\d{4,8}$/.test(pin) ? t('users.pinInvalid') : undefined}
              value={pin}
              onChange={(e) => setPin(e.target.value.replace(/\D/g, ''))}
            />
            {user?.has_pin && (
              <label className="flex cursor-pointer items-center gap-2 text-label text-muted">
                <input type="checkbox" checked={removePin} onChange={(e) => setRemovePin(e.target.checked)} className="h-4 w-4 accent-[rgb(var(--primary))]" />
                {t('users.removePin')}
              </label>
            )}
          </div>
        )}

        {!isAdmin && (
          <div className="flex flex-col gap-2">
            <span className="text-label font-medium">{t('users.access')}</span>
            <div className="grid grid-cols-2 gap-2">
              {[false, true].map((v) => (
                <button
                  key={String(v)}
                  type="button"
                  aria-pressed={limited === v}
                  onClick={() => setLimited(v)}
                  className={cn('cursor-pointer rounded-md border p-3 text-start text-label transition-colors', limited === v ? 'border-primary bg-primary-container text-on-primary-container' : 'border-border text-muted hover:border-border-strong')}
                >
                  <span className="block font-medium">{t(v ? 'users.accessSome' : 'users.accessAll')}</span>
                  <span className="mt-0.5 block text-caption opacity-80">{t(v ? 'users.accessSomeHint' : 'users.accessAllHint')}</span>
                </button>
              ))}
            </div>
            {limited && (
              <div className="flex flex-col rounded-md border border-border p-1">
                {libraries?.map((l) => (
                  <label key={l.id} className="flex min-h-9 cursor-pointer items-center gap-3 rounded px-2 hover:bg-raised">
                    <input
                      type="checkbox"
                      checked={libraryIds.includes(l.id)}
                      onChange={(e) => setLibraryIds(e.target.checked ? [...libraryIds, l.id] : libraryIds.filter((x) => x !== l.id))}
                      className="h-4 w-4 accent-[rgb(var(--primary))]"
                    />
                    <span className="text-body">{l.name}</span>
                  </label>
                ))}
              </div>
            )}
          </div>
        )}
        {!isAdmin && (
          <fieldset className="flex flex-col gap-3">
            <legend className="mb-1 text-label font-medium">{t('household.title')}</legend>
            <div className="flex flex-col gap-1.5">
              <label htmlFor="household-limit" className="text-label">
                {t('household.limit')}
              </label>
              <Select
                id="household-limit"
                aria-describedby="household-limit-hint"
                value={prefs.max_rating ?? ''}
                onChange={(e) => setPrefs({ ...prefs, max_rating: e.target.value === '' ? null : Number(e.target.value) })}
              >
                <option value="">{t('household.noLimit')}</option>
                {(mine?.options.rating_levels ?? []).map((age) => (
                  <option key={age} value={age}>
                    {ratingLabel(age)}
                  </option>
                ))}
              </Select>
              <p id="household-limit-hint" className="text-caption text-muted">
                {t('household.limitHint')}
              </p>
            </div>
            <div className="-mx-4 divide-y divide-border border-y border-border">
              {prefs.max_rating != null && (
                <SettingRow label={t('household.unrated')} description={t('household.unratedHint')}>
                  <Switch checked={prefs.allow_unrated} onChange={(v) => setPrefs({ ...prefs, allow_unrated: v })} label={t('household.unrated')} />
                </SettingRow>
              )}
              <SettingRow label={t('household.lanUnlock')} description={willHavePin ? t('household.lanUnlockHint') : t('household.lanUnlockNeedsPin')}>
                <Switch checked={prefs.lan_unlock && willHavePin} onChange={(v) => willHavePin && setPrefs({ ...prefs, lan_unlock: v })} label={t('household.lanUnlock')} />
              </SettingRow>
            </div>
          </fieldset>
        )}
        {mine && (
          <fieldset className="flex flex-col gap-2">
            <legend className="mb-1 text-label font-medium">{t('profile.languages')}</legend>
            <p className="text-caption text-muted">{t('household.languagesHint')}</p>
            <LanguageFields idPrefix="edit" value={prefs} onChange={(l) => setPrefs({ ...prefs, ...l })} options={mine.options} />
          </fieldset>
        )}
        {user && user.id === me?.id && <p className="text-caption text-subtle">{t('users.editingSelf')}</p>}
        <ErrorText>{save.error && errorText(save.error)}</ErrorText>
      </form>
    </Modal>
  )
}

export function UsersSection() {
  const qc = useQueryClient()
  const { user: me } = useAuth()
  const { data: users, isLoading } = useQuery({ queryKey: ['users'], queryFn: api.users })
  const { data: libraries } = useLibraries()
  const [editing, setEditing] = useState<ManagedUser | null | 'new'>(null)
  const [transfer, setTransfer] = useState<ManagedUser | null>(null)
  const remove = useMutation({
    mutationFn: api.deleteUser,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['users'] })
      qc.invalidateQueries({ queryKey: ['profiles'] })
    },
  })
  const libraryNames = (ids: number[]) => ids.map((id) => libraries?.find((l) => l.id === id)?.name).filter(Boolean).join(', ')

  return (
    <Card>
      <CardHeader
        title={t('users.title')}
        action={
          <Button size="sm" variant="primary" icon={<Plus size={16} />} onClick={() => setEditing('new')}>
            {t('users.add')}
          </Button>
        }
      />
      {isLoading && <Skeleton className="m-4 h-24" />}
      <ul className="divide-y divide-border">
        {users?.map((u) => (
          <li key={u.id} className="flex items-center gap-4 px-4 py-3">
            <Avatar name={u.name} color={u.avatar} size={40} />
            <div className="min-w-0 flex-1">
              <p className="flex flex-wrap items-center gap-2 text-body font-medium">
                {u.name}
                {u.id === me?.id && <span className="text-caption text-subtle">{t('users.you')}</span>}
                {u.is_owner && (
                  <Badge tone="primary">
                    <Crown size={12} weight="fill" aria-hidden />
                    {t('users.owner')}
                  </Badge>
                )}
                {u.is_admin ? (
                  <Badge tone="primary">
                    <ShieldCheck size={12} weight="fill" />
                    {t('users.admin')}
                  </Badge>
                ) : (
                  <Badge>{t('users.member')}</Badge>
                )}
                {(u.has_pin || u.has_password) && (
                  <Badge>
                    <LockSimple size={12} weight="fill" />
                    {t(u.has_password ? 'users.lockPassword' : 'users.lockPin')}
                  </Badge>
                )}
                {!u.is_admin && u.preferences.max_rating != null && (
                  <Badge tone="warn">
                    <ShieldStar size={12} weight="fill" aria-hidden />
                    {ratingLabel(u.preferences.max_rating)}
                  </Badge>
                )}
                {!u.is_admin && u.preferences.lan_unlock && u.has_pin && (
                  <Badge>
                    <HouseLine size={12} weight="fill" aria-hidden />
                    {t('household.lanUnlock')}
                  </Badge>
                )}
              </p>
              <p className="truncate text-caption text-muted">{u.library_ids.length > 0 ? t('users.limitedTo', { names: libraryNames(u.library_ids) }) : t('users.accessAll')}</p>
            </div>
            <Button size="sm" variant="ghost" aria-label={t('transfer.open', { name: u.name })} title={t('transfer.title')} icon={<ArrowsLeftRight size={16} />} onClick={() => setTransfer(u)} />
            <Button size="sm" variant="ghost" aria-label={t('users.edit')} title={t('users.edit')} icon={<PencilSimple size={16} />} disabled={u.is_owner && u.id !== me?.id} onClick={() => setEditing(u)} />
            <Button
              size="sm"
              variant="ghost"
              disabled={u.id === me?.id || u.is_owner}
              aria-label={t('users.delete')}
              title={t('users.delete')}
              icon={<Trash size={16} />}
              onClick={() => window.confirm(t('users.deleteConfirm', { name: u.name })) && remove.mutate(u.id)}
            />
          </li>
        ))}
      </ul>
      {remove.error && (
        <div className="p-4">
          <ErrorText>{errorText(remove.error)}</ErrorText>
        </div>
      )}
      {/* key remounts the editor so each open starts from that profile's values. */}
      {editing && <ProfileEditor key={editing === 'new' ? 'new' : editing.id} user={editing === 'new' ? null : editing} open onClose={() => setEditing(null)} />}
      {transfer && (
        <TransferModal
          open
          onClose={() => setTransfer(null)}
          name={transfer.name}
          withLock
          onExport={(includeLock) => api.exportUser(transfer.id, includeLock)}
          onImport={async (data, applyLock) => {
            const r = await api.importUser(transfer.id, data, applyLock)
            qc.invalidateQueries({ queryKey: ['users'] })
            qc.invalidateQueries({ queryKey: ['profiles'] })
            return r
          }}
        />
      )}
    </Card>
  )
}
