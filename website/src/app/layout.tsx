import { type Metadata, type Viewport } from 'next'
import localFont from 'next/font/local'
import clsx from 'clsx'

import { Providers } from '@/app/providers'
import { Layout } from '@/components/Layout'

import '@/styles/tailwind.css'

const sans = localFont({
  src: '../fonts/lexend.woff2',
  display: 'swap',
  variable: '--font-inter',
})

// Use local version of Lexend so that we can use OpenType features
const lexend = localFont({
  src: '../fonts/lexend.woff2',
  display: 'swap',
  variable: '--font-lexend',
})

const siteUrl = 'https://pharos-cli.io'
const siteDescription =
  'Pharos traces vulnerable JavaScript dependencies through yarn.lock and package-lock.json files, then shows which top-level package owns the fix.'

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  applicationName: 'Pharos',
  title: {
    template: '%s — Pharos',
    default: 'Pharos — Trace vulnerable JavaScript dependencies through lockfiles',
  },
  description: siteDescription,
  keywords: [
    'Pharos',
    'pharos-cli',
    'dependency security',
    'npm audit',
    'yarn audit',
    'package-lock.json',
    'yarn.lock',
    'JavaScript dependencies',
    'vulnerability remediation',
  ],
  authors: [{ name: 'Vasilis Kortsimelidis', url: 'https://vaskort.com' }],
  creator: 'Vasilis Kortsimelidis',
  publisher: 'Vasilis Kortsimelidis',
  alternates: {
    canonical: '/',
  },
  openGraph: {
    type: 'website',
    url: siteUrl,
    title: 'Pharos — Trace vulnerable JavaScript dependencies',
    description: siteDescription,
    siteName: 'Pharos',
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
    title: 'Pharos — Trace vulnerable JavaScript dependencies',
    description: siteDescription,
    images: ['/og.png'],
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-image-preview': 'large',
      'max-snippet': -1,
      'max-video-preview': -1,
    },
  },
  icons: {
    icon: '/favicon.svg',
  },
  manifest: '/site.webmanifest',
  category: 'developer tool',
}

export const viewport: Viewport = {
  themeColor: '#0f172a',
  colorScheme: 'dark light',
}

const structuredData = {
  '@context': 'https://schema.org',
  '@type': 'SoftwareApplication',
  name: 'Pharos',
  alternateName: 'pharos-cli',
  applicationCategory: 'DeveloperApplication',
  operatingSystem: 'macOS, Linux, Windows',
  url: siteUrl,
  codeRepository: 'https://github.com/vaskort/pharos',
  downloadUrl: 'https://www.npmjs.com/package/pharos-cli',
  description: siteDescription,
  author: {
    '@type': 'Person',
    name: 'Vasilis Kortsimelidis',
    url: 'https://vaskort.com',
  },
  offers: {
    '@type': 'Offer',
    price: '0',
    priceCurrency: 'USD',
  },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html
      lang="en"
      className={clsx('h-full antialiased', sans.variable, lexend.variable)}
      suppressHydrationWarning
    >
      <body className="flex min-h-full bg-white dark:bg-slate-900">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(structuredData) }}
        />
        <Providers>
          <Layout>{children}</Layout>
        </Providers>
      </body>
    </html>
  )
}
