import { type Metadata } from 'next'

const siteUrl = 'https://pharos-cli.io'

export function createDocMetadata({
  title,
  description,
  path,
}: {
  title: string
  description: string
  path: string
}): Metadata {
  return {
    title,
    description,
    alternates: {
      canonical: path,
    },
    openGraph: {
      type: 'website',
      title: `${title} — Pharos`,
      description,
      siteName: 'Pharos',
      url: new URL(path, siteUrl),
      images: [
        {
          url: '/og.png',
          width: 1200,
          height: 630,
          alt: 'Pharos — Trace vulnerable JavaScript dependencies',
        },
      ],
    },
    twitter: {
      card: 'summary_large_image',
      title: `${title} — Pharos`,
      description,
      images: ['/og.png'],
    },
  }
}
