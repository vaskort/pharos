'use client'

import { useEffect, useMemo, useState } from 'react'
import Link from 'next/link'
import clsx from 'clsx'

import { type Section, type Subsection } from '@/lib/sections'

function getSectionIds(tableOfContents: Array<Section>) {
  return tableOfContents.flatMap((section) => [
    section.id,
    ...section.children.map((child) => child.id),
  ])
}

export function TableOfContents({
  tableOfContents,
}: {
  tableOfContents: Array<Section>
}) {
  let sectionIds = useMemo(
    () => getSectionIds(tableOfContents),
    [tableOfContents],
  )
  let [currentSection, setCurrentSection] = useState(sectionIds[0])

  useEffect(() => {
    if (sectionIds.length === 0) {
      return
    }

    let headings = sectionIds
      .map((id) => document.getElementById(id))
      .filter((heading): heading is HTMLElement => heading !== null)

    if (headings.length === 0) {
      return
    }

    let frameId = 0

    function updateCurrentSection() {
      frameId = 0

      let current = headings[0].id
      let viewportTop = window.innerHeight * 0.25

      for (let heading of headings) {
        if (heading.getBoundingClientRect().top <= viewportTop) {
          current = heading.id
        } else {
          break
        }
      }

      setCurrentSection(current)
    }

    function scheduleUpdate() {
      if (frameId === 0) {
        frameId = window.requestAnimationFrame(updateCurrentSection)
      }
    }

    let observer = new IntersectionObserver(scheduleUpdate, {
      rootMargin: '-25% 0px -70% 0px',
    })

    for (let heading of headings) {
      observer.observe(heading)
    }

    window.addEventListener('scroll', scheduleUpdate, { passive: true })
    window.addEventListener('resize', scheduleUpdate)
    updateCurrentSection()

    return () => {
      observer.disconnect()
      window.removeEventListener('scroll', scheduleUpdate)
      window.removeEventListener('resize', scheduleUpdate)

      if (frameId !== 0) {
        window.cancelAnimationFrame(frameId)
      }
    }
  }, [sectionIds])

  function isActive(section: Section | Subsection) {
    if (section.id === currentSection) {
      return true
    }
    if (!section.children) {
      return false
    }
    return section.children.findIndex(isActive) > -1
  }

  return (
    <div className="hidden xl:sticky xl:top-19 xl:-mr-6 xl:block xl:h-[calc(100vh-4.75rem)] xl:flex-none xl:overflow-y-auto xl:py-16 xl:pr-6">
      <nav aria-labelledby="on-this-page-title" className="w-56">
        {tableOfContents.length > 0 && (
          <>
            <h2
              id="on-this-page-title"
              className="font-display text-sm font-medium text-slate-900 dark:text-white"
            >
              On this page
            </h2>
            <ol role="list" className="mt-4 space-y-3 text-sm">
              {tableOfContents.map((section) => (
                <li key={section.id}>
                  <h3>
                    <Link
                      href={`#${section.id}`}
                      className={clsx(
                        isActive(section)
                          ? 'text-teal-500'
                          : 'font-normal text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-300',
                      )}
                    >
                      {section.title}
                    </Link>
                  </h3>
                  {section.children.length > 0 && (
                    <ol
                      role="list"
                      className="mt-2 space-y-3 pl-5 text-slate-500 dark:text-slate-400"
                    >
                      {section.children.map((subSection) => (
                        <li key={subSection.id}>
                          <Link
                            href={`#${subSection.id}`}
                            className={
                              isActive(subSection)
                                ? 'text-teal-500'
                                : 'hover:text-slate-600 dark:hover:text-slate-300'
                            }
                          >
                            {subSection.title}
                          </Link>
                        </li>
                      ))}
                    </ol>
                  )}
                </li>
              ))}
            </ol>
          </>
        )}
      </nav>
    </div>
  )
}
