import {
	ArrowLeft,
	ArrowRight,
	BookOpen,
	FileText,
	Info,
	Loader2,
	Plus,
	RefreshCw,
	Search,
	Trash2,
	Upload,
	X,
} from "lucide-react";
import { type FC, useEffect, useRef, useState } from "react";
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { Textarea } from "@/components/ui/textarea";
import { loadKnowledgeBasesIntoGraph } from "@/entities/fetchers/knowledge";
import { useKnowledgeAdmin } from "@/hooks/use-knowledge-admin";
import { useOnboarding } from "@/hooks/use-onboarding";
import { cn, friendlyError } from "@/lib/utils";
import type { UarKnowledgeBase, UarKnowledgeDocument } from "@/types";

const STATUS_VARIANT: Record<string, string> = {
	indexed: "bg-emerald-500/15 text-emerald-400 border-emerald-500/25",
	pending: "bg-amber-500/15 text-amber-400 border-amber-500/25",
	processing: "bg-blue-500/15 text-blue-400 border-blue-500/25",
	failed: "bg-destructive/15 text-destructive border-destructive/25",
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const KnowledgePage: FC = () => {
	const {
		bases,
		loading,
		error,
		saving,
		deleting,
		docs,
		docsLoading,
		docsError,
		uploading,
		uploadProgress,
		searchResults,
		searching,
		deletingDoc,
		loadBases,
		addBase,
		removeBase,
		loadDocs,
		uploadFiles,
		removeDocument,
		runSearch,
		clearSearch,
		clearDocView,
		setUploadProgress,
	} = useKnowledgeAdmin();

	// Populate the entity graph alongside the legacy store.
	useEffect(() => {
		void loadKnowledgeBasesIntoGraph();
	}, []);

	const [showAdd, setShowAdd] = useState(false);
	const [form, setForm] = useState({ name: "", description: "" });

	const [deleteTarget, setDeleteTarget] = useState<UarKnowledgeBase | null>(
		null,
	);

	const [selectedKb, setSelectedKb] = useState<UarKnowledgeBase | null>(null);

	const fileInputRef = useRef<HTMLInputElement>(null);

	const [deleteDocTarget, setDeleteDocTarget] =
		useState<UarKnowledgeDocument | null>(null);

	const [searchQuery, setSearchQuery] = useState("");

	const { dismissed: pipelineDismissed, dismiss: dismissPipeline } =
		useOnboarding("kb-pipeline");

	const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

	const handleAdd = async () => {
		try {
			await addBase(form);
			setShowAdd(false);
			setForm({ name: "", description: "" });
		} catch {
			/* store sets error */
		}
	};

	const handleDeleteKb = async () => {
		if (!deleteTarget) return;
		try {
			await removeBase(deleteTarget.id);
			if (selectedKb?.id === deleteTarget.id) {
				setSelectedKb(null);
				clearDocView();
			}
			setDeleteTarget(null);
		} catch {
			/* swallow */
		}
	};

	useEffect(() => {
		if (!selectedKb) return;
		const hasPending = docs.some(
			(d) => d.status === "pending" || d.status === "processing",
		);
		if (hasPending) {
			pollRef.current = setInterval(() => {
				void loadDocs(selectedKb.id);
			}, 3000);
		}
		return () => {
			if (pollRef.current) clearInterval(pollRef.current);
		};
	}, [docs, selectedKb, loadDocs]);

	const selectKb = (kb: UarKnowledgeBase) => {
		setSelectedKb(kb);
		clearSearch();
		setSearchQuery("");
		void loadDocs(kb.id);
	};

	const handleUpload = async (files: FileList | null) => {
		if (!files || !selectedKb) return;
		setUploadProgress(`Uploading ${files.length} file(s)...`);
		try {
			await uploadFiles(selectedKb.id, files);
		} catch {
			/* store sets docsError */
		} finally {
			if (fileInputRef.current) fileInputRef.current.value = "";
		}
	};

	const handleDeleteDoc = async () => {
		if (!deleteDocTarget || !selectedKb) return;
		try {
			await removeDocument(selectedKb.id, deleteDocTarget);
			setDeleteDocTarget(null);
		} catch {
			/* swallow */
		}
	};

	const handleSearch = async () => {
		if (!selectedKb || !searchQuery.trim()) return;
		await runSearch(selectedKb.id, searchQuery);
	};

	// =========================================================================
	// Drag & drop
	// =========================================================================

	const [dragOver, setDragOver] = useState(false);

	const onDrop = (e: React.DragEvent) => {
		e.preventDefault();
		setDragOver(false);
		void handleUpload(e.dataTransfer.files);
	};

	// =========================================================================
	// Render: KB list view
	// =========================================================================

	const renderKbList = () => (
		<div className="flex flex-1 flex-col overflow-hidden">
			{/* Header */}
			<div className="flex items-center justify-between border-b border-border bg-card px-4 py-4 sm:px-6">
				<div>
					<h2 className="font-display text-lg font-semibold text-foreground">
						Knowledge Bases
					</h2>
					<p className="font-mono text-xs text-muted-foreground">
						{bases.length} {bases.length === 1 ? "base" : "bases"}
					</p>
				</div>
				<div className="flex gap-2">
					<Button
						variant="outline"
						size="sm"
						onClick={() => void loadBases()}
						className="gap-1.5"
						aria-label="Refresh knowledge bases"
					>
						<RefreshCw size={13} className={cn(loading && "animate-spin")} />
						<span className="hidden sm:inline">Refresh</span>
					</Button>
					<Button
						size="sm"
						onClick={() => setShowAdd(true)}
						className="gap-1.5"
					>
						<Plus size={13} />
						<span className="hidden sm:inline">New</span>
					</Button>
				</div>
			</div>

			{/* Content */}
			<div className="flex-1 overflow-y-auto p-4 sm:p-6">
				{loading && (
					<div className="flex items-center gap-2 py-8 justify-center">
						<Loader2 size={16} className="animate-spin text-muted-foreground" />
						<span className="font-mono text-xs text-muted-foreground">
							Loading...
						</span>
					</div>
				)}
				{error && (
					<p className="font-mono text-xs text-destructive">
						{friendlyError(error)}
					</p>
				)}
				{!loading && bases.length === 0 && !error && (
					<div className="flex flex-col items-center gap-3 py-16 text-center">
						<div className="flex size-12 items-center justify-center rounded-xl bg-primary/10">
							<BookOpen size={20} className="text-primary" />
						</div>
						<p className="font-display text-sm font-medium text-foreground">
							No knowledge bases yet
						</p>
						<p className="max-w-xs font-body text-xs text-muted-foreground">
							Create a knowledge base to store documents and search them by
							meaning, not just keywords.
						</p>
						<Button
							size="sm"
							onClick={() => setShowAdd(true)}
							className="mt-2 gap-1.5"
						>
							<Plus size={13} />
							Create Knowledge Base
						</Button>
					</div>
				)}
				<div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
					{bases.map((kb) => (
						<button
							key={kb.id}
							type="button"
							onClick={() => selectKb(kb)}
							className="flex flex-col rounded-lg border border-border bg-card p-4 text-left transition-colors hover:border-primary/40 hover:bg-card/80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
						>
							<div className="mb-3 flex items-start justify-between">
								<div className="flex size-8 items-center justify-center rounded-md bg-primary/15">
									<BookOpen size={14} className="text-primary" />
								</div>
								<Button
									variant="ghost"
									size="icon"
									className="h-6 w-6 text-muted-foreground hover:text-destructive"
									onClick={(e) => {
										e.stopPropagation();
										setDeleteTarget(kb);
									}}
									aria-label={`Delete ${kb.name}`}
								>
									<Trash2 size={12} />
								</Button>
							</div>
							<p className="font-display text-sm font-semibold text-foreground">
								{kb.name}
							</p>
							{kb.description && (
								<p className="mt-1 font-body text-xs text-muted-foreground line-clamp-2">
									{kb.description}
								</p>
							)}
							<p className="mt-2 font-mono text-xs text-muted-foreground">
								{kb.document_count ?? 0} documents
							</p>
						</button>
					))}
				</div>
			</div>
		</div>
	);

	// =========================================================================
	// Render: Document detail view
	// =========================================================================

	const renderDocDetail = () => {
		if (!selectedKb) return null;
		return (
			<section
				aria-label="Knowledge base documents and uploads"
				className="flex flex-1 flex-col overflow-hidden"
				onDragOver={(e) => {
					e.preventDefault();
					setDragOver(true);
				}}
				onDragLeave={() => setDragOver(false)}
				onDrop={onDrop}
			>
				{/* Header */}
				<div className="flex items-center justify-between border-b border-border bg-card px-4 py-4 sm:px-6">
					<div className="flex items-center gap-3 min-w-0">
						<Button
							variant="ghost"
							size="icon"
							className="h-7 w-7 shrink-0"
							onClick={() => {
								setSelectedKb(null);
								clearDocView();
								setSearchQuery("");
							}}
							aria-label="Back to knowledge bases"
						>
							<ArrowLeft size={14} />
						</Button>
						<div className="min-w-0">
							<h2 className="truncate font-display text-lg font-semibold text-foreground">
								{selectedKb.name}
							</h2>
							<p className="font-mono text-xs text-muted-foreground">
								{docs.length} {docs.length === 1 ? "document" : "documents"}
							</p>
						</div>
					</div>
					<div className="flex gap-2 shrink-0">
						<Button
							variant="outline"
							size="sm"
							onClick={() => void loadDocs(selectedKb.id)}
							className="gap-1.5"
							aria-label="Refresh documents"
						>
							<RefreshCw
								size={13}
								className={cn(docsLoading && "animate-spin")}
							/>
							<span className="hidden sm:inline">Refresh</span>
						</Button>
						<Button
							type="button"
							size="sm"
							onClick={() => fileInputRef.current?.click()}
							disabled={uploading}
							className="gap-1.5"
							aria-controls="kb-upload-input"
						>
							{uploading ? (
								<Loader2 size={13} className="animate-spin" />
							) : (
								<Upload size={13} />
							)}
							<span className="hidden sm:inline">Upload</span>
						</Button>
						<input
							id="kb-upload-input"
							ref={fileInputRef}
							type="file"
							multiple
							className="hidden"
							aria-label="Select files to upload to this knowledge base"
							title="Select files to upload"
							onChange={(e) => void handleUpload(e.target.files)}
							accept=".pdf,.txt,.md,.csv,.json,.html,.xml,.doc,.docx,.xls,.xlsx,.pptx,.rtf"
						/>
					</div>
				</div>

				{/* Search bar */}
				<div className="flex items-center gap-2 border-b border-border bg-card/50 px-4 py-3 sm:px-6">
					<div className="relative flex-1">
						<Search
							size={14}
							className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
						/>
						<Input
							value={searchQuery}
							onChange={(e) => setSearchQuery(e.target.value)}
							onKeyDown={(e) => {
								if (e.key === "Enter") void handleSearch();
							}}
							placeholder="Search by meaning across all documents..."
							className="pl-9 font-mono text-xs"
						/>
					</div>
					<Button
						variant="outline"
						size="sm"
						onClick={() => void handleSearch()}
						disabled={searching || !searchQuery.trim()}
						className="gap-1.5 shrink-0"
					>
						{searching ? (
							<Loader2 size={13} className="animate-spin" />
						) : (
							<Search size={13} />
						)}
						Search
					</Button>
					{searchResults !== null && (
						<Button
							variant="ghost"
							size="sm"
							onClick={() => clearSearch()}
							className="text-xs"
						>
							Clear
						</Button>
					)}
				</div>

				{/* Upload progress */}
				{uploadProgress && (
					<div className="border-b border-border bg-blue-500/5 px-4 py-2 sm:px-6">
						<div className="flex items-center gap-2">
							<Loader2 size={12} className="animate-spin text-blue-400" />
							<span className="font-mono text-xs text-blue-400">
								{uploadProgress}
							</span>
						</div>
						<Progress value={undefined} className="mt-1.5 h-1" />
					</div>
				)}

				{/* Pipeline guidance — first visit */}
				{!pipelineDismissed && (
					<div className="border-b border-border bg-primary/5 px-4 py-3 sm:px-6">
						<div className="flex items-start gap-3">
							<Info size={14} className="mt-0.5 shrink-0 text-primary" />
							<div className="min-w-0 flex-1">
								<p className="font-display text-sm font-medium text-foreground">
									How documents are processed
								</p>
								<div className="mt-2 flex flex-wrap items-center gap-1.5 font-mono text-xs text-muted-foreground">
									<span className="rounded bg-amber-500/15 px-2 py-0.5 text-amber-400">
										Upload
									</span>
									<ArrowRight size={10} className="text-muted-foreground/50" />
									<span className="rounded bg-blue-500/15 px-2 py-0.5 text-blue-400">
										Processing
									</span>
									<ArrowRight size={10} className="text-muted-foreground/50" />
									<span className="rounded bg-emerald-500/15 px-2 py-0.5 text-emerald-400">
										Indexed
									</span>
									<ArrowRight size={10} className="text-muted-foreground/50" />
									<span className="rounded bg-primary/15 px-2 py-0.5 text-primary">
										Searchable
									</span>
								</div>
								<p className="mt-1.5 font-body text-xs leading-relaxed text-muted-foreground">
									Uploaded files are split into chunks, converted to embeddings,
									and indexed for search. Processing happens in the background —
									status updates automatically.
								</p>
							</div>
							<button
								type="button"
								onClick={dismissPipeline}
								className="shrink-0 rounded p-1 text-muted-foreground transition-colors hover:text-foreground"
								aria-label="Dismiss pipeline guidance"
							>
								<X size={12} />
							</button>
						</div>
					</div>
				)}

				{/* Drag overlay */}
				{dragOver && (
					<div className="absolute inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
						<div className="flex flex-col items-center gap-3 rounded-xl border-2 border-dashed border-primary/50 bg-card p-12">
							<Upload size={32} className="text-primary" />
							<p className="font-display text-sm font-medium text-foreground">
								Drop files to upload
							</p>
						</div>
					</div>
				)}

				{/* Content */}
				<div className="flex-1 overflow-y-auto p-4 sm:p-6">
					{docsError && (
						<p className="mb-3 font-mono text-xs text-destructive">
							{friendlyError(docsError)}
						</p>
					)}

					{/* Search results */}
					{searchResults !== null ? (
						<div className="space-y-3">
							<p className="font-mono text-xs text-muted-foreground">
								{searchResults.length} result
								{searchResults.length !== 1 ? "s" : ""} for &ldquo;
								{searchQuery}&rdquo;
							</p>
							{searchResults.length === 0 && (
								<p className="py-8 text-center font-body text-xs text-muted-foreground">
									No matches found. Try a different query.
								</p>
							)}
							{searchResults.map((r) => (
								<div
									key={`${r.document_id ?? "doc"}-${r.score}-${r.content.slice(0, 48)}`}
									className="rounded-lg border border-border bg-card p-4"
								>
									<div className="mb-2 flex items-center justify-between">
										<Badge variant="outline" className="font-mono text-xs">
											Score: {(r.score * 100).toFixed(1)}%
										</Badge>
										{r.document_id && (
											<span className="font-mono text-xs text-muted-foreground">
												doc: {r.document_id.slice(0, 8)}
											</span>
										)}
									</div>
									<p className="whitespace-pre-wrap font-body text-xs leading-relaxed text-foreground">
										{r.content}
									</p>
								</div>
							))}
						</div>
					) : (
						<>
							{/* Document list */}
							{docsLoading && docs.length === 0 && (
								<div className="flex items-center gap-2 py-8 justify-center">
									<Loader2
										size={16}
										className="animate-spin text-muted-foreground"
									/>
									<span className="font-mono text-xs text-muted-foreground">
										Loading documents...
									</span>
								</div>
							)}
							{!docsLoading && docs.length === 0 && !docsError && (
								<div className="flex flex-col items-center gap-3 py-16 text-center">
									<div className="flex size-12 items-center justify-center rounded-xl bg-muted">
										<FileText size={20} className="text-muted-foreground" />
									</div>
									<p className="font-display text-sm font-medium text-foreground">
										No documents yet
									</p>
									<p className="max-w-xs font-body text-xs text-muted-foreground">
										Upload files to index them into this knowledge base.
										Supported formats include PDF, text, Markdown, CSV, JSON,
										and Office documents.
									</p>
									<Button
										size="sm"
										onClick={() => fileInputRef.current?.click()}
										className="mt-2 gap-1.5"
									>
										<Upload size={13} />
										Upload Files
									</Button>
								</div>
							)}
							<div className="space-y-2">
								{docs.map((doc) => (
									<div
										key={doc.id}
										className="flex items-center gap-3 rounded-lg border border-border bg-card p-3 sm:p-4"
									>
										<div className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted">
											<FileText size={14} className="text-muted-foreground" />
										</div>
										<div className="min-w-0 flex-1">
											<p className="truncate font-display text-sm font-medium text-foreground">
												{doc.filename}
											</p>
											<div className="mt-1 flex flex-wrap items-center gap-2">
												<Badge
													variant="outline"
													className={cn(
														"font-mono text-xs border",
														STATUS_VARIANT[doc.status] ??
															"text-muted-foreground",
													)}
												>
													{doc.status === "processing" && (
														<Loader2 size={10} className="mr-1 animate-spin" />
													)}
													{doc.status}
												</Badge>
												{doc.chunk_count > 0 && (
													<span className="font-mono text-xs text-muted-foreground">
														{doc.chunk_count} chunks
													</span>
												)}
												{doc.mime_type && (
													<span className="hidden font-mono text-xs text-muted-foreground sm:inline">
														{doc.mime_type}
													</span>
												)}
											</div>
											{doc.status === "failed" && doc.error_message && (
												<p className="mt-1 font-mono text-xs text-destructive line-clamp-1">
													{doc.error_message}
												</p>
											)}
										</div>
										<Button
											variant="ghost"
											size="icon"
											className="h-7 w-7 shrink-0 text-muted-foreground hover:text-destructive"
											onClick={() => setDeleteDocTarget(doc)}
											aria-label={`Delete ${doc.filename}`}
										>
											<Trash2 size={12} />
										</Button>
									</div>
								))}
							</div>
						</>
					)}
				</div>
			</section>
		);
	};

	// =========================================================================
	// Render
	// =========================================================================

	return (
		<div className="relative flex flex-1 flex-col overflow-hidden">
			{selectedKb ? renderDocDetail() : renderKbList()}

			{/* ---- Create KB Dialog ---- */}
			<Dialog open={showAdd} onOpenChange={setShowAdd}>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Add Knowledge Base</DialogTitle>
					</DialogHeader>
					<div className="flex flex-col gap-3">
						<div>
							<Label
								htmlFor="kb-new-name"
								className="mb-1 block font-mono text-xs text-muted-foreground"
							>
								Name
							</Label>
							<Input
								id="kb-new-name"
								value={form.name}
								onChange={(e) => setForm({ ...form, name: e.target.value })}
								placeholder="My Knowledge Base"
							/>
						</div>
						<div>
							<Label
								htmlFor="kb-new-description"
								className="mb-1 block font-mono text-xs text-muted-foreground"
							>
								Description
							</Label>
							<Textarea
								id="kb-new-description"
								value={form.description}
								onChange={(e) =>
									setForm({ ...form, description: e.target.value })
								}
								placeholder="Optional description..."
								rows={3}
							/>
						</div>
					</div>
					<DialogFooter>
						<Button variant="ghost" onClick={() => setShowAdd(false)}>
							Cancel
						</Button>
						<Button
							onClick={() => void handleAdd()}
							disabled={saving || !form.name.trim()}
						>
							{saving && <Loader2 size={14} className="animate-spin" />}
							Create
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>

			{/* ---- Delete KB AlertDialog ---- */}
			<AlertDialog
				open={deleteTarget !== null}
				onOpenChange={(open) => !open && setDeleteTarget(null)}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Delete knowledge base?</AlertDialogTitle>
						<AlertDialogDescription>
							This will permanently delete <strong>{deleteTarget?.name}</strong>{" "}
							and all its documents and search data. This action cannot be
							undone.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
						<AlertDialogAction
							onClick={(e) => {
								e.preventDefault();
								void handleDeleteKb();
							}}
							disabled={deleting}
							className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
						>
							{deleting ? (
								<Loader2 size={12} className="animate-spin" />
							) : (
								"Delete"
							)}
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>

			{/* ---- Delete Document AlertDialog ---- */}
			<AlertDialog
				open={deleteDocTarget !== null}
				onOpenChange={(open) => !open && setDeleteDocTarget(null)}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Delete document?</AlertDialogTitle>
						<AlertDialogDescription>
							This will permanently delete{" "}
							<strong>{deleteDocTarget?.filename}</strong> and all its processed
							content. This action cannot be undone.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel disabled={deletingDoc}>Cancel</AlertDialogCancel>
						<AlertDialogAction
							onClick={(e) => {
								e.preventDefault();
								void handleDeleteDoc();
							}}
							disabled={deletingDoc}
							className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
						>
							{deletingDoc ? (
								<Loader2 size={12} className="animate-spin" />
							) : (
								"Delete"
							)}
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</div>
	);
};
