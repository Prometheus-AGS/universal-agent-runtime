import type { Meta, StoryObj } from "@storybook/react-vite";
import { Button } from "./button";
import { Badge } from "./badge";
import { Alert, AlertTitle, AlertDescription } from "./alert";
import { Avatar, AvatarFallback, AvatarImage } from "./avatar";
import { Switch } from "./switch";
import { Spinner } from "./spinner";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./tabs";
import { Textarea } from "./textarea";
import { Skeleton } from "./skeleton";
import { Slider } from "./slider";
import { Separator } from "./separator";
import { Card, CardHeader, CardTitle, CardContent } from "./card";
import { Progress } from "./progress";
import { Toggle } from "./toggle";
import { Input } from "./input";
import { Label } from "./label";
import { Kbd } from "./kbd";
import { Checkbox } from "./checkbox";

/**
 * The shadcn/ui visual baseline `@prometheus-ags/a2ui-uar` components are
 * built on (`frontend/components.json`). Not exhaustive -- a representative
 * slice of the 56-component library covering the primitives A2UI
 * components actually compose (button, input, card, etc.).
 */
const meta: Meta = { title: "UI Primitives" };
export default meta;
type Story = StoryObj;

export const ButtonPrimary: Story = {
  render: () => <Button>Confirm</Button>,
};
export const ButtonOutline: Story = { render: () => <Button variant="outline">Cancel</Button> };
export const BadgeDefault: Story = {
  render: () => <Badge>New</Badge>,
};
export const AlertDefault: Story = {
  render: () => (
    <Alert>
      <AlertTitle>Heads up</AlertTitle>
      <AlertDescription>Your session expires in 5 minutes.</AlertDescription>
    </Alert>
  ),
};
export const AvatarDefault: Story = {
  render: () => (
    <Avatar>
      <AvatarImage src="https://placehold.co/40x40" alt="User avatar" />
      <AvatarFallback>AB</AvatarFallback>
    </Avatar>
  ),
};
export const SwitchDefault: Story = { render: () => <Switch aria-label="Enable notifications" /> };
export const SpinnerDefault: Story = { render: () => <Spinner /> };
export const TabsDefault: Story = {
  render: () => (
    <Tabs defaultValue="one">
      <TabsList>
        <TabsTrigger value="one">One</TabsTrigger>
        <TabsTrigger value="two">Two</TabsTrigger>
      </TabsList>
      <TabsContent value="one">First tab content</TabsContent>
      <TabsContent value="two">Second tab content</TabsContent>
    </Tabs>
  ),
};
export const TextareaDefault: Story = { render: () => <Textarea placeholder="Notes" /> };
export const SkeletonDefault: Story = { render: () => <Skeleton className="h-4 w-32" /> };
export const SliderDefault: Story = {
  render: () => <Slider defaultValue={[40]} max={100} aria-label="Volume" />,
};
export const SeparatorDefault: Story = { render: () => <Separator /> };
export const CardDefault: Story = {
  render: () => (
    <Card>
      <CardHeader>
        <CardTitle>Order #123</CardTitle>
      </CardHeader>
      <CardContent>Placed 2026-07-10</CardContent>
    </Card>
  ),
};
export const ProgressDefault: Story = { render: () => <Progress value={60} aria-label="Upload progress" /> };
export const ToggleDefault: Story = { render: () => <Toggle aria-label="Toggle bold">B</Toggle> };
export const InputDefault: Story = { render: () => <Input placeholder="Name" /> };
export const LabelDefault: Story = {
  render: () => (
    <div>
      <Label htmlFor="story-input">Name</Label>
      <Input id="story-input" />
    </div>
  ),
};
export const KbdDefault: Story = {
  render: () => <Kbd>⌘K</Kbd>,
};
export const CheckboxDefault: Story = { render: () => <Checkbox aria-label="Accept terms" /> };

export const LightThemeContrast: Story = {
  render: () => (
    <div className="light flex flex-wrap items-center gap-4 rounded-lg bg-background p-6 text-foreground">
      <Button>Confirm</Button>
      <Badge>New</Badge>
      <Avatar>
        <AvatarFallback>AB</AvatarFallback>
      </Avatar>
      <Kbd>Ctrl K</Kbd>
      <Tabs defaultValue="one">
        <TabsList>
          <TabsTrigger value="one">Active</TabsTrigger>
          <TabsTrigger value="two">Inactive</TabsTrigger>
        </TabsList>
        <TabsContent value="one">Light theme accessibility baseline</TabsContent>
      </Tabs>
    </div>
  ),
};
