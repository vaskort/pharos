import { createDocMetadata } from '@/lib/seo'

export const metadata = createDocMetadata({
  title: 'Understanding output',
  description:
    'Learn how to read Pharos dependency chains, package owners, fix paths, and recommended parent upgrades.',
  path: '/docs/understanding-output/',
})

export default function UnderstandingOutputLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
