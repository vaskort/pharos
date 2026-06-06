import { createDocMetadata } from '@/lib/seo'

export const metadata = createDocMetadata({
  title: 'Limitations',
  description:
    'Understand the current Pharos scope, unsupported lockfiles, workspace limitations, and registry-based fix suggestions.',
  path: '/docs/limitations/',
})

export default function LimitationsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
