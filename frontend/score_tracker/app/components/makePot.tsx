import { useState } from "react"

export interface MakePotProps {
  onConfirm: (name: string, description: string, score_requirement: number) => void
}
export default function MakePot({ onConfirm }: MakePotProps) {
  const [name, setName] = useState("")
  const [description, setDescription] = useState("")
  const [scoreReq, setScoreReq] = useState(0)
  function reset() {
    setName("")
    setDescription("")
    setScoreReq(0)
  }
  return (
    <div>
      <label>Name:
        <input onChange={(ev) => setName(ev.target.value)} />
      </label>
      <label>Description:
        <input onChange={(ev) => setDescription(ev.target.value)} />
      </label>
      <label>Score Requirement:
        <input onChange={(ev) => setScoreReq(parseInt(ev.target.value))} />
      </label>
      <button onClick={() => {
        reset()
        onConfirm(name, description, scoreReq)
      }}>Confirm</button>
      <button onClick={() => {
        reset()
      }}>Cancel</button>
    </div>
  )

}
