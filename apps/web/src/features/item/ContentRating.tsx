import { useState, type FormEvent } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { ShieldStar } from '@phosphor-icons/react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { Field } from '@/components/ui/Field'
import { Modal } from '@/components/ui/Modal'
import { api } from '@/lib/api'
import { t } from '@/lib/i18n'
import type { ItemCard } from '@/lib/types'

/** Admin: set a title's certification by hand, for files TMDB does not rate. */
export function ContentRatingButton({ item }: { item: ItemCard }) {
  const qc = useQueryClient()
  const [open, setOpen] = useState(false)
  const [value, setValue] = useState(item.content_rating ?? '')
  const save = useMutation({
    mutationFn: () => api.setContentRating(item.id, value.trim() || null),
    onSuccess: () => {
      qc.invalidateQueries()
      setOpen(false)
    },
  })
  const submit = (e: FormEvent) => {
    e.preventDefault()
    save.mutate()
  }
  return (
    <>
      <Button icon={<ShieldStar size={16} />} onClick={() => setOpen(true)}>
        {item.content_rating ? `${t('rating.label')}: ${item.content_rating}` : t('rating.edit')}
      </Button>
      <Modal
        open={open}
        onClose={() => setOpen(false)}
        title={t('rating.edit')}
        footer={
          <>
            <Button variant="ghost" onClick={() => setOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button variant="primary" type="submit" form="rating-form" loading={save.isPending}>
              {t('common.save')}
            </Button>
          </>
        }
      >
        <form id="rating-form" onSubmit={submit} className="flex flex-col gap-3">
          <Field label={t('rating.label')} hint={t('rating.hint')} maxLength={16} value={value} onChange={(e) => setValue(e.target.value)} />
          <ErrorText>{save.error?.message}</ErrorText>
        </form>
      </Modal>
    </>
  )
}
