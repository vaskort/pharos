import { createDocMetadata } from '@/lib/seo'

export const metadata = createDocMetadata({
  title: 'Supported lockfiles',
  description:
    'See which JavaScript lockfile formats Pharos can parse, including yarn.lock and package-lock.json v2/v3.',
  path: '/docs/lockfiles/',
})

export default function SupportedLockfilesLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
