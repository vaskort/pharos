import { createDocMetadata } from '@/lib/seo'

export const metadata = createDocMetadata({
  title: 'CLI reference',
  description:
    'Command syntax, options, and examples for scanning JavaScript lockfiles with the Pharos CLI.',
  path: '/docs/cli/',
})

export default function CliReferenceLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
