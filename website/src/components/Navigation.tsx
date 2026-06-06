import Link from 'next/link'
import { usePathname } from 'next/navigation'
import clsx from 'clsx'

import { navigation } from '@/lib/navigation'

function normalizePathname(pathname: string) {
  let normalized = pathname || '/'

  if (normalized !== '/' && normalized.endsWith('/')) {
    normalized = normalized.slice(0, -1)
  }

  return normalized
}

export function Navigation({
  className,
  onLinkClick,
}: {
  className?: string
  onLinkClick?: React.MouseEventHandler<HTMLAnchorElement>
}) {
  let pathname = normalizePathname(usePathname())

  return (
    <nav className={clsx('text-base lg:text-sm', className)}>
      <ul role="list" className="space-y-9">
        {navigation.map((section) => (
          <li key={section.title}>
            <h2 className="font-display font-medium text-slate-900 dark:text-white">
              {section.title}
            </h2>
            <ul
              role="list"
              className="mt-2 space-y-2 border-l-2 border-slate-100 lg:mt-4 lg:space-y-4 lg:border-slate-200 dark:border-slate-800"
            >
              {section.links.map((link) => {
                let isActive = normalizePathname(link.href) === pathname

                return (
                  <li key={link.href} className="relative">
                    <Link
                      href={link.href}
                      onClick={onLinkClick}
                      aria-current={isActive ? 'page' : undefined}
                      className={clsx(
                        'relative block w-full rounded-r-md py-1.5 pr-3 pl-4 transition before:pointer-events-none before:absolute before:inset-y-1 before:-left-px before:w-0.5 before:rounded-r-full',
                        isActive
                          ? 'bg-teal-50 font-semibold text-teal-700 before:bg-teal-500 dark:bg-teal-400/10 dark:text-teal-300 dark:before:bg-teal-300'
                          : 'text-slate-500 before:bg-transparent hover:bg-slate-100/70 hover:text-slate-700 hover:before:bg-slate-300 dark:text-slate-400 dark:hover:bg-slate-800/70 dark:hover:text-slate-200 dark:hover:before:bg-slate-600',
                      )}
                    >
                      {link.title}
                    </Link>
                  </li>
                )
              })}
            </ul>
          </li>
        ))}
      </ul>
    </nav>
  )
}
