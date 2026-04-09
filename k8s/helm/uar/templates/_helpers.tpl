{{/*
Expand the name of the chart.
*/}}
{{- define "uar.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "uar.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "uar.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "uar.labels" -}}
helm.sh/chart: {{ include "uar.chart" . }}
{{ include "uar.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: universal-agent-runtime
{{- end }}

{{/*
Selector labels
*/}}
{{- define "uar.selectorLabels" -}}
app.kubernetes.io/name: {{ include "uar.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
UAR app component labels
*/}}
{{- define "uar.appLabels" -}}
{{ include "uar.labels" . }}
app.kubernetes.io/component: app
{{- end }}

{{/*
UAR app selector labels
*/}}
{{- define "uar.appSelectorLabels" -}}
app.kubernetes.io/name: uar
app.kubernetes.io/component: app
{{- end }}

{{/*
Namespace name
*/}}
{{- define "uar.namespace" -}}
{{- default .Release.Namespace .Values.namespaceOverride }}
{{- end }}
