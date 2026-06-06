import { createDocMetadata } from '@/lib/seo'

export const metadata = createDocMetadata({
  title: 'JSON reports',
  description:
    'Use Pharos JSON output in CI and downstream tools to attach dependency-chain context to security findings.',
  path: '/docs/ci-json/',
})

export default function JsonReportsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
