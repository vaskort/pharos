import Image from 'next/image'

import { AnimatedPharosOutput } from '@/components/AnimatedPharosOutput'
import { Button } from '@/components/Button'
import pharosHero from '@/images/pharos-hero.svg'

function TrafficLightsIcon(props: React.ComponentPropsWithoutRef<'svg'>) {
  return (
    <svg aria-hidden="true" viewBox="0 0 42 10" fill="none" {...props}>
      <circle cx="5" cy="5" r="4.5" />
      <circle cx="21" cy="5" r="4.5" />
      <circle cx="37" cy="5" r="4.5" />
    </svg>
  )
}

export function Hero() {
  return (
    <div className="overflow-hidden bg-slate-900 dark:-mt-19 dark:-mb-32 dark:pt-19 dark:pb-32">
      <div className="py-16 sm:px-2 lg:relative lg:px-0 lg:py-24">
        <div className="mx-auto grid max-w-2xl grid-cols-1 items-center gap-x-12 gap-y-12 px-4 lg:max-w-7xl lg:grid-cols-[1.1fr_1fr] lg:px-8 xl:gap-x-16 xl:px-12">
          <div className="relative z-10 md:text-center lg:text-left">
            <h1 className="font-display text-4xl tracking-tight text-white sm:text-5xl">
              Why is that vulnerable package in your lockfile?
            </h1>
            <p className="mt-5 max-w-xl text-lg leading-relaxed text-slate-300 md:mx-auto lg:mx-0">
              Pharos walks{' '}
              <code className="rounded bg-slate-800 px-1.5 py-0.5 text-base text-slate-200">
                yarn.lock
              </code>{' '}
              or{' '}
              <code className="rounded bg-slate-800 px-1.5 py-0.5 text-base text-slate-200">
                package-lock.json
              </code>{' '}
              upward from a known vulnerable{' '}
              <code className="rounded bg-slate-800 px-1.5 py-0.5 text-base text-slate-200">
                package@version
              </code>{' '}
              and tells you which top-level dependency owns the fix.
            </p>
            <div className="mt-8 flex gap-4 md:justify-center lg:justify-start">
              <Button href="#run-without-installing">Get started</Button>
              <Button
                href="https://github.com/vaskort/pharos"
                variant="secondary"
              >
                View on GitHub
              </Button>
            </div>
          </div>
          <div className="relative">
            <Image
              src={pharosHero}
              alt=""
              priority
              className="pointer-events-none absolute -top-24 -right-12 hidden h-72 w-auto opacity-90 lg:block xl:-top-28 xl:-right-4 xl:h-80"
            />
            <div className="relative rounded-2xl bg-slate-950/80 ring-1 ring-white/10 backdrop-blur-sm">
              <div className="px-4 pt-4">
                <TrafficLightsIcon className="h-2.5 w-auto stroke-slate-500/40" />
              </div>
              <AnimatedPharosOutput />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
