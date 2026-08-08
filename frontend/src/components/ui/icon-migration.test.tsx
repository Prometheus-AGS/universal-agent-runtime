import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, test, vi } from "vitest"

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group"
import {
  Sheet,
  SheetContent,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet"

describe("Base UI icon migration semantics", () => {
  test("keeps the dialog close icon accessible and functional", async () => {
    const user = userEvent.setup()
    render(
      <Dialog>
        <DialogTrigger render={<button type="button" />}>Open settings</DialogTrigger>
        <DialogContent>
          <DialogTitle>Settings</DialogTitle>
        </DialogContent>
      </Dialog>
    )

    await user.click(screen.getByRole("button", { name: "Open settings" }))
    expect(screen.getByRole("dialog", { name: "Settings" })).toBeVisible()

    await user.click(screen.getByRole("button", { name: "Close" }))
    expect(screen.queryByRole("dialog", { name: "Settings" })).not.toBeInTheDocument()
  })

  test("keeps the sheet close icon accessible and functional", async () => {
    const user = userEvent.setup()
    render(
      <Sheet>
        <SheetTrigger render={<button type="button" />}>Open filters</SheetTrigger>
        <SheetContent>
          <SheetTitle>Filters</SheetTitle>
        </SheetContent>
      </Sheet>
    )

    await user.click(screen.getByRole("button", { name: "Open filters" }))
    expect(screen.getByRole("dialog", { name: "Filters" })).toBeVisible()

    await user.click(screen.getByRole("button", { name: "Close" }))
    expect(screen.queryByRole("dialog", { name: "Filters" })).not.toBeInTheDocument()
  })

  test("keeps accordion chevrons tied to expanded state", async () => {
    const user = userEvent.setup()
    render(
      <Accordion>
        <AccordionItem value="details">
          <AccordionTrigger>Details</AccordionTrigger>
          <AccordionContent>Runtime details</AccordionContent>
        </AccordionItem>
      </Accordion>
    )

    const trigger = screen.getByRole("button", { name: "Details" })
    expect(trigger).toHaveAttribute("aria-expanded", "false")

    await user.click(trigger)
    expect(trigger).toHaveAttribute("aria-expanded", "true")
    expect(screen.getByText("Runtime details")).toBeVisible()
  })

  test("keeps the Lucide check icon tied to checkbox state", async () => {
    const user = userEvent.setup()
    render(<Checkbox aria-label="Enable alerts" />)

    const checkbox = screen.getByRole("checkbox", { name: "Enable alerts" })
    expect(checkbox).not.toBeChecked()

    await user.click(checkbox)
    expect(checkbox).toBeChecked()
  })

  test("keeps the native radio indicator tied to selection state", async () => {
    const user = userEvent.setup()
    const onValueChange = vi.fn()
    render(
      <RadioGroup aria-label="Runtime mode" onValueChange={onValueChange}>
        <RadioGroupItem value="local" aria-label="Local" />
        <RadioGroupItem value="remote" aria-label="Remote" />
      </RadioGroup>
    )

    const remote = screen.getByRole("radio", { name: "Remote" })
    await user.click(remote)

    expect(remote).toBeChecked()
    expect(onValueChange).toHaveBeenCalledWith("remote", expect.anything())
  })
})
