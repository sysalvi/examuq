<?php

namespace App\Http\Middleware;

use App\Models\LaunchToken;
use Carbon\Carbon;
use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class EnsureValidLaunchToken
{
    public function handle(Request $request, Closure $next): Response
    {
        $rawToken = $request->query('lt');

        if (! is_string($rawToken) || trim($rawToken) === '') {
            return redirect()->route('exam.block');
        }

        $hash = hash('sha256', $rawToken);

        $launchToken = LaunchToken::query()
            ->with(['exam', 'session'])
            ->where('token_hash', $hash)
            ->first();

        if (! $launchToken) {
            return redirect()->route('exam.block');
        }

        $now = Carbon::now();

        if ($launchToken->expires_at->lte($now) || ! $launchToken->exam?->is_active) {
            return redirect()->route('exam.block');
        }

        $request->attributes->set('launchToken', $launchToken);
        $request->attributes->set('exam', $launchToken->exam);
        $request->attributes->set('examSession', $launchToken->session);

        return $next($request);
    }
}
