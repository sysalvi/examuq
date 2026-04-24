<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Exam;
use App\Models\ExamSession;
use App\Models\LaunchToken;
use Carbon\Carbon;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Str;

class LaunchController extends Controller
{
    public function requestLaunch(Request $request): JsonResponse
    {
        $payload = $request->validate([
            'exam_id' => ['nullable', 'integer', 'exists:exams,id'],
            'display_name' => ['required', 'string', 'max:255'],
            'class_room' => ['required', 'string', 'max:255'],
            'token_global' => ['required', 'string', 'max:255'],
            'client_type' => ['required', 'string', 'in:desktop_client,chrome_extension'],
            'device_id' => ['nullable', 'string', 'max:255'],
        ]);

        $exam = null;
        if (isset($payload['exam_id']) && $payload['exam_id']) {
            $exam = Exam::query()->findOrFail((int) $payload['exam_id']);
        } else {
            $exam = Exam::query()
                ->where('token_global', $payload['token_global'])
                ->where('is_active', true)
                ->orderByDesc('id')
                ->first();
        }

        if (! $exam) {
            return response()->json([
                'message' => 'Token tidak valid atau ujian tidak aktif.',
            ], 422);
        }

        if (! $exam->is_active || $exam->token_global !== $payload['token_global']) {
            return response()->json([
                'message' => 'Token tidak valid atau ujian tidak aktif.',
            ], 422);
        }

        $resolvedClientType = (string) ($request->attributes->get('examuqClientType') ?: $payload['client_type']);
        $resolvedDeviceId = $request->attributes->get('examuqDeviceId') ?: ($payload['device_id'] ?? null);

        if (! in_array($resolvedClientType, ['desktop_client', 'chrome_extension'], true)) {
            $resolvedClientType = $payload['client_type'];
        }

        $now = Carbon::now();

        $session = ExamSession::query()->create([
            'exam_id' => $exam->id,
            'display_name' => $payload['display_name'],
            'class_room' => $payload['class_room'],
            'client_type' => $resolvedClientType,
            'device_id' => is_string($resolvedDeviceId) && $resolvedDeviceId !== '' ? $resolvedDeviceId : null,
            'status' => 'active',
            'started_at' => $now,
            'last_heartbeat_at' => $now,
            'ip_address' => $request->ip(),
            'user_agent' => $request->userAgent(),
        ]);

        $rawToken = Str::random(64);
        $expiresAt = $now->copy()->addMinutes(5);

        LaunchToken::query()->create([
            'exam_id' => $exam->id,
            'exam_session_id' => $session->id,
            'token_hash' => hash('sha256', $rawToken),
            'issued_for_client' => $resolvedClientType,
            'issued_for_ip' => $request->ip(),
            'payload_json' => [
                'display_name' => $payload['display_name'],
                'class_room' => $payload['class_room'],
            ],
            'expires_at' => $expiresAt,
        ]);

        return response()->json([
            'message' => 'Launch token berhasil dibuat.',
            'session_id' => $session->id,
            'launch_token' => $rawToken,
            'expires_at' => $expiresAt->toIso8601String(),
            'redirect_url' => url('/exam-gateway?lt='.$rawToken),
        ]);
    }
}
