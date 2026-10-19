import type { AvatarColor } from '@/lib/types'

/** Fixed hues that read on both themes; the server keeps the same list. */
export const avatarColors: Record<AvatarColor, string> = {
  ember: 'linear-gradient(135deg, #E8553A, #9C2F1C)',
  ocean: 'linear-gradient(135deg, #3B82F6, #1E3A8A)',
  forest: 'linear-gradient(135deg, #22A06B, #135C3D)',
  plum: 'linear-gradient(135deg, #A855F7, #5B2192)',
  sun: 'linear-gradient(135deg, #F5B12A, #B26B05)',
  rose: 'linear-gradient(135deg, #EC4899, #9D174D)',
  slate: 'linear-gradient(135deg, #64748B, #2B3442)',
  teal: 'linear-gradient(135deg, #14B8A6, #0F5E57)',
}
