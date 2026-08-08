import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { useForm } from "react-hook-form"
import { describe, expect, test, vi } from "vitest"

import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"

type TestFormValues = {
  email: string
}

function TestForm({ onSubmit }: { onSubmit: (values: TestFormValues) => void }) {
  const form = useForm<TestFormValues>({
    defaultValues: { email: "" },
  })

  return (
    <Form {...form}>
      <form noValidate onSubmit={form.handleSubmit(onSubmit)}>
        <FormField
          control={form.control}
          name="email"
          rules={{
            required: "Email is required",
            pattern: {
              value: /^[^@]+@[^@]+$/,
              message: "Enter a valid email",
            },
          }}
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input type="email" {...field} />
              </FormControl>
              <FormDescription>Used for runtime notifications.</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <button type="submit">Save</button>
      </form>
    </Form>
  )
}

describe("Form", () => {
  test("connects Field semantics to React Hook Form validation and submission", async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()

    render(<TestForm onSubmit={onSubmit} />)

    const input = screen.getByRole("textbox", { name: "Email" })
    expect(input).toHaveAccessibleDescription("Used for runtime notifications.")

    await user.click(screen.getByRole("button", { name: "Save" }))
    expect(await screen.findByText("Email is required")).toBeVisible()
    expect(input).toHaveAttribute("aria-invalid", "true")
    expect(onSubmit).not.toHaveBeenCalled()

    await user.type(input, "operator@example.com")
    await user.click(screen.getByRole("button", { name: "Save" }))

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith(
        { email: "operator@example.com" },
        expect.anything()
      )
    })
  })
})
