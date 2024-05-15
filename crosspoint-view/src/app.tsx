import React, { useState, useCallback } from 'react'
import { createRoot } from 'react-dom/client'
import layout from './layout.json'

type StatusType = { [coords: string]: boolean }
type Coords = [number, number]

function lookupSwitch(status: StatusType, coords: Coords): boolean {
  return Boolean(status[`${coords[0]}.${coords[1]}`])
}

function targetRef(id: string, dimension: string, index: number): string {
  let chip = null
  for (const c of layout.chips) {
    if (c.chip === id) {
      chip = c
      break
    }
  }
  if (!chip) {
    throw new Error(`Unknown chip: ${id}`)
  }
  const target = (dimension === 'X' ? chip.x_map : chip.y_map)[index]
  if (target.Chip) {
    return `${target.Chip[0]}${target.Chip[1].toLowerCase()}${target.Chip[2]}`
  } else {
    return `<${target.Node}>`
  }
}

const Crosspoint: React.FC<{ id: string, status: StatusType, onChange: (coords: Coords, value: boolean) => void }> = ({ id, status, onChange }) => {
  const xCount = 16
  const yCount = 8

  const rows = []
  for (let y = 0; y < yCount; y++) {
    const cols = []
    for (let x = 0; x < xCount; x++) {
      cols.push(<td key={x}><input type='checkbox' name={name} checked={lookupSwitch(status, [x, y])} onChange={e => onChange([x, y], e.currentTarget.checked)} /></td>)
    }
    rows.push((
      <tr key={y}>
        <th title={targetRef(id, 'Y', y)}>
          y{y}
        </th>
        {cols}
      </tr>
    ))
  }
  const headers = []
  for (let x = 0; x < xCount; x++) {
    headers.push(
      <th key={x} title={targetRef(id, 'X', x)}>
        x{x}
      </th>
    )
  }

  return (
    <div className='Crosspoint'>
      <h1>{id}</h1>
      <table>
        <thead><tr><th />{headers}</tr></thead>
        <tbody>{rows}</tbody>
      </table>
    </div>
  )
}

function hexByte(byte: number): string {
  return byte.toString(16).padStart(2, '0')
}

function chipStatusToString(status: StatusType): string {
  let result = ''
  for (let x = 0; x < 16; x++) {
    let byte = 0
    for (let y = 0; y < 8; y++) {
      if (status[`${x}.${y}`]) {
        byte |= 1 << y
      }
    }
    result += hexByte(byte)
  }
  return result
}

function statusToString(status: { [chip: string]: StatusType }): string {
  let result = ''
  for (const chip in status) {
    result += `${chip}:${chipStatusToString(status[chip])}\n`
  }
  return result
}

const App: React.FC = () => {
  const [status, setStatus] = useState({
    A: {},
    B: {},
    C: {},
    D: {},
    E: {},
    F: {},
    G: {},
    H: {},
    I: {},
    J: {},
    K: {},
    L: {},
  })
  const handleChange = useCallback((id: string, coords: Coords, state: boolean) => {
    setStatus({
      ...status,
      [id]: {
        ...status[id],
        [`${coords[0]}.${coords[1]}`]: state,
      },
    })
  }, [status])
  return (
    <>
      <div className='Crosspoints'>
        {['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'].map(id => (
          <Crosspoint key={id} id={id} status={status[id]} onChange={(coords, state) => handleChange(id, coords, state)} />
        ))}
      </div>
      <pre>{statusToString(status)}</pre>
    </>
  )
}

const root = createRoot(document.getElementById('root'))
root.render(<App />)

