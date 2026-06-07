'use client'

import { useEffect, useState } from 'react'

const command = '$ npx pharos-cli@latest qs@6.13.0 --path ./my-app'

const outputLines = [
  '',
  '  ./package-lock.json',
  '  Found qs@6.13.0',
  '',
  '  Owner: express, requested as ^4.18.0',
  '',
  '  Chain',
  '    qs@6.13.0',
  '      -> body-parser@1.20.3',
  '      -> express@4.21.2',
  '',
  '  Fix path',
  '    body-parser >= 1.20.4',
  '    express    >= 5.0.0',
  '',
]

const recommendedLine = '→ Recommended: update express to >= 5.0.0'

const initialAnimationState = {
  commandLength: 0,
  visibleLineCount: 0,
  showRecommendation: false,
}

function TerminalCursor() {
  return (
    <span
      aria-hidden="true"
      className="ml-0.5 inline-block h-4 w-2 translate-y-0.5 animate-pulse rounded-sm bg-teal-300"
    />
  )
}

export function AnimatedPharosOutput() {
  const [animationState, setAnimationState] = useState(initialAnimationState)

  useEffect(() => {
    let timeoutId: number

    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      timeoutId = window.setTimeout(() => {
        setAnimationState({
          commandLength: command.length,
          visibleLineCount: outputLines.length,
          showRecommendation: true,
        })
      }, 0)

      return () => {
        window.clearTimeout(timeoutId)
      }
    }

    let commandLength = 0
    let visibleLineCount = 0

    function typeCommand() {
      commandLength += 1
      setAnimationState({
        commandLength,
        visibleLineCount,
        showRecommendation: false,
      })

      if (commandLength < command.length) {
        timeoutId = window.setTimeout(typeCommand, 22)
        return
      }

      timeoutId = window.setTimeout(revealNextLine, 280)
    }

    function revealNextLine() {
      visibleLineCount += 1
      setAnimationState({
        commandLength,
        visibleLineCount,
        showRecommendation: false,
      })

      if (visibleLineCount < outputLines.length) {
        timeoutId = window.setTimeout(revealNextLine, 150)
        return
      }

      timeoutId = window.setTimeout(() => {
        setAnimationState({
          commandLength,
          visibleLineCount,
          showRecommendation: true,
        })
      }, 220)
    }

    timeoutId = window.setTimeout(typeCommand, 360)

    return () => {
      window.clearTimeout(timeoutId)
    }
  }, [])

  const commandText = command.slice(0, animationState.commandLength)
  const visibleLines = outputLines.slice(0, animationState.visibleLineCount)
  const isTypingCommand = animationState.commandLength < command.length
  const isRevealingOutput =
    animationState.commandLength === command.length &&
    animationState.visibleLineCount < outputLines.length

  return (
    <pre
      aria-label={`${command}\n${outputLines.join('\n')}${recommendedLine}`}
      className="min-h-[30rem] overflow-x-auto px-5 pt-3 pb-5 font-mono text-[13px] leading-6 text-slate-200"
    >
      <span className="text-slate-100">{commandText}</span>
      {isTypingCommand && <TerminalCursor />}
      {'\n'}
      {visibleLines.map((line, index) => (
        <span
          key={`${index}-${line}`}
          className="block animate-[terminal-line_180ms_ease-out_both]"
        >
          {line || '\u00a0'}
        </span>
      ))}
      {isRevealingOutput && <TerminalCursor />}
      {animationState.showRecommendation && (
        <span className="block animate-[terminal-line_220ms_ease-out_both] text-amber-300">
          {recommendedLine}
        </span>
      )}
    </pre>
  )
}
