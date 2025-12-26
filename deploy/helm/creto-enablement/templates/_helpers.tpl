{{/*
Expand the name of the chart.
*/}}
{{- define "creto-enablement.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "creto-enablement.fullname" -}}
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
{{- define "creto-enablement.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "creto-enablement.labels" -}}
helm.sh/chart: {{ include "creto-enablement.chart" . }}
{{ include "creto-enablement.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: creto-enablement
{{- end }}

{{/*
Selector labels
*/}}
{{- define "creto-enablement.selectorLabels" -}}
app.kubernetes.io/name: {{ include "creto-enablement.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "creto-enablement.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "creto-enablement.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Return the proper image name
*/}}
{{- define "creto-enablement.image" -}}
{{- $registryName := .global.imageRegistry -}}
{{- $repositoryName := .image.repository -}}
{{- $tag := .image.tag | default "latest" -}}
{{- printf "%s/%s:%s" $registryName $repositoryName $tag -}}
{{- end }}

{{/*
Return Redis host
*/}}
{{- define "creto-enablement.redis.host" -}}
{{- if .Values.redis.enabled }}
{{- printf "%s-redis-master" (include "creto-enablement.fullname" .) }}
{{- else }}
{{- .Values.externalRedis.host }}
{{- end }}
{{- end }}

{{/*
Return Redis port
*/}}
{{- define "creto-enablement.redis.port" -}}
{{- if .Values.redis.enabled }}
{{- .Values.redis.master.service.ports.redis | default 6379 }}
{{- else }}
{{- .Values.externalRedis.port | default 6379 }}
{{- end }}
{{- end }}

{{/*
Return Redis secret name
*/}}
{{- define "creto-enablement.redis.secretName" -}}
{{- if .Values.redis.enabled }}
{{- printf "%s-redis" (include "creto-enablement.fullname" .) }}
{{- else }}
{{- .Values.externalRedis.existingSecret }}
{{- end }}
{{- end }}

{{/*
Return Redis secret key
*/}}
{{- define "creto-enablement.redis.secretKey" -}}
{{- if .Values.redis.enabled }}
{{- "redis-password" }}
{{- else }}
{{- .Values.externalRedis.existingSecretPasswordKey | default "redis-password" }}
{{- end }}
{{- end }}

{{/*
Return PostgreSQL host
*/}}
{{- define "creto-enablement.postgresql.host" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "creto-enablement.fullname" .) }}
{{- else }}
{{- .Values.externalPostgresql.host }}
{{- end }}
{{- end }}

{{/*
Return PostgreSQL port
*/}}
{{- define "creto-enablement.postgresql.port" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.primary.service.ports.postgresql | default 5432 }}
{{- else }}
{{- .Values.externalPostgresql.port | default 5432 }}
{{- end }}
{{- end }}

{{/*
Return PostgreSQL database name
*/}}
{{- define "creto-enablement.postgresql.database" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.database | default "creto_enablement" }}
{{- else }}
{{- .Values.externalPostgresql.database }}
{{- end }}
{{- end }}

{{/*
Return PostgreSQL secret name
*/}}
{{- define "creto-enablement.postgresql.secretName" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "creto-enablement.fullname" .) }}
{{- else }}
{{- .Values.externalPostgresql.existingSecret }}
{{- end }}
{{- end }}
