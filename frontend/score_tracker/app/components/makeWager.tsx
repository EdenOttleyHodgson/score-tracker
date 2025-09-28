import { useState } from "react"
import type { WagerOutcome } from "~/backend/types"

export interface MakeWagerProps {
  onConfirm: (name: string, outcomes: WagerOutcome[]) => void
}



export default function MakeWager({ onConfirm }: MakeWagerProps) {
  const [name, setName] = useState("")
  const [outcomes, setOutcomes] = useState<WagerOutcome[]>([])
  const outcomeEntries = outcomes.map((outcome) => {
    return (
      <OutcomeEntry
        outcome={outcome}
        onChange={(newOutcome) => {
          setOutcomes(outcomes.map((toChange) => {
            if (toChange === outcome) {
              return newOutcome
            } else {
              return toChange
            }
          }))
        }}
        onRemove={() =>
          setOutcomes(outcomes.filter((toRemove) => toRemove !== outcome))}
      >
      </OutcomeEntry>
    )
  })


  return (
    <div>
      <label>
        Wager Name:
        <input defaultValue={name} onChange={(ev) => setName(ev.target.value)}></input>
      </label>
      <div>
        {outcomeEntries}
        <button onClick={() => {
          setOutcomes([...outcomes, { id: outcomes.length, name: "", description: "", odds: 0 }])
        }}>Add Outcome</button>
      </div>
      <button onClick={() => {
        setOutcomes([])
        setName("")
        onConfirm(name, outcomes)
      }}>Confirm</button>

    </div>)

}

function OutcomeEntry({ outcome, onChange, onRemove }: { outcome: WagerOutcome, onChange: (arg0: WagerOutcome) => void, onRemove: () => void }) {
  return (<div>
    <label>
      Name
      <input defaultValue={outcome.name} onChange={(ev) => onChange({ ...outcome, name: ev.target.value })} />
    </label>
    <label>
      Description
      <input defaultValue={outcome.description} onChange={(ev) => onChange({ ...outcome, description: ev.target.value })} /> </label>
    <label>
      Odds
      <input min={0} max={100} type={"number"} defaultValue={outcome.odds} onChange={(ev) => onChange({ ...outcome, odds: parseInt(ev.target.value) })} />
    </label>
    <button onClick={onRemove}>Remove</button>
  </div>)


}
